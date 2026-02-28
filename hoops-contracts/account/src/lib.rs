#![no_std]

use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contractimpl, contracttype, contracterror, crypto::Hash, symbol_short,
    token::Client as TokenClient, Address, Bytes, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

mod base64_url;
mod verify;
#[cfg(test)]
mod test;

pub mod hoops_router {
    soroban_sdk::contractimport!(
        file = "../bytecodes/hoops_router.wasm"
    );
    pub type RouterClient<'a> = Client<'a>;
}
use hoops_router::RouterClient;

// Define LpPlan locally so the Account WASM embeds the full struct spec.
// contractimport! generates the type but doesn't re-export field definitions,
// which breaks CLI argument parsing. This is wire-compatible with the Router's LpPlan.
// Fields must be in alphabetical order for Soroban contracttype serialization.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LpPlan {
    pub adapter_id: i128,
    pub amount_a: i128,
    pub amount_b: i128,
    pub pool_address: Address,
    pub token_a: Address,
    pub token_b: Address,
}

// RedeemPlan: per-pool withdrawal instruction for batch_redeem.
// No warehouse flag — Account holds all LP directly (auth tree validated).
// Fields must be in alphabetical order for Soroban contracttype serialization.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedeemPlan {
    pub adapter_id: i128,
    pub lp_amount: i128,
    pub lp_token: Address,
    pub pool_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Secp256r1Signature {
    pub authenticator_data: Bytes,
    pub client_data_json: Bytes,
    pub signature: BytesN<64>,
}

#[contracttype]
#[derive(Clone)]
pub enum AccountSignature {
    Passkey(Secp256r1Signature),
    Session(BytesN<64>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccountError {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    PasskeyNotSet = 3,
    ClientDataJsonChallengeIncorrect = 4,
    JsonParseError = 5,
    NoActiveSession = 6,
    SessionExpired = 7,
    SessionScopeViolation = 8,
}

#[contracttype]
#[derive(Clone)]
enum Key { Owner, Router, PasskeyPubkey, SessionKey }

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionKey {
    pub pubkey: BytesN<32>,
    pub scope: Vec<Symbol>,
    pub max_amount: i128,
    pub expiry_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct TokenEvent { pub token: Address, pub amount: i128 }

#[contract]
pub struct Account;

#[contractimpl]
impl Account {
    /* ---- lifecycle ---- */
    pub fn initialize(e: Env, owner: Address, router: Address) -> Result<(), AccountError> {
        if e.storage().instance().has(&Key::Owner) {
            return Err(AccountError::AlreadyInitialized)
        }
        owner.require_auth();
        e.storage().instance().set(&Key::Owner, &owner);
        e.storage().instance().set(&Key::Router, &router);
        Ok(())
    }
    pub fn upgrade(e: Env, wasm: BytesN<32>) -> Result<(), AccountError> {
        Self::owner(&e).require_auth();
        e.deployer().update_current_contract_wasm(wasm);
        Ok(())
    }

    /* ---- token passthrough ---- */
    pub fn transfer(e: Env, token: Address, to: Address, amount: i128) -> Result<(), AccountError> {
        Self::owner(&e).require_auth();
        TokenClient::new(&e,&token)
            .transfer(&e.current_contract_address(), &to, &amount);
        e.events().publish(("acct", symbol_short!("xfer")),
                           TokenEvent{ token, amount });
        Ok(())
    }

    /* ---- liquidity flow  (tokens already in Account) ---- */
    pub fn deposit(
        e: Env,
        usdc: Address,
        amount: i128,
        lp_plans: Vec<LpPlan>,
        deadline: u32,
    ) -> Result<(), AccountError> {
        let owner = Self::owner(&e);
        owner.require_auth();

        let router = Self::router(&e);

        // Transfer both tokens from Account to Router for each LP plan.
        // Account is direct caller → auth works (same pattern as swap).
        for plan in lp_plans.iter() {
            TokenClient::new(&e, &plan.token_a)
                .transfer(&e.current_contract_address(), &router, &plan.amount_a);
            TokenClient::new(&e, &plan.token_b)
                .transfer(&e.current_contract_address(), &router, &plan.amount_b);
        }

        // Use invoke_contract instead of typed client to avoid LpPlan type mismatch
        // (local LpPlan vs contractimport-generated LpPlan). Both are XDR-identical.
        let mut args: Vec<Val> = Vec::new(&e);
        args.push_back(amount.into_val(&e));
        args.push_back(lp_plans.into_val(&e));
        args.push_back(e.current_contract_address().into_val(&e));
        args.push_back((deadline as u64).into_val(&e));
        let _: () = e.invoke_contract(
            &router,
            &Symbol::new(&e, "provide_liquidity"),
            args,
        );

        e.events().publish(("acct", symbol_short!("dep")),
            TokenEvent{ token: usdc, amount });
        Ok(())
    }

    pub fn batch_redeem(
        e: Env,
        plans: Vec<RedeemPlan>,
        deadline: u32,
    ) -> Result<(), AccountError> {
        let owner = Self::owner(&e);
        owner.require_auth();
        let router = Self::router(&e);
        let plan_count = plans.len();

        // Transfer LP tokens from Account to Router for ALL plans.
        // Account holds LP for all protocols (no warehouse asymmetry).
        for plan in plans.iter() {
            TokenClient::new(&e, &plan.lp_token)
                .transfer(&e.current_contract_address(), &router, &plan.lp_amount);
        }

        // Single Router call with all plans
        let mut args: Vec<Val> = Vec::new(&e);
        args.push_back(plans.into_val(&e));
        args.push_back(e.current_contract_address().into_val(&e));
        args.push_back((deadline as u64).into_val(&e));
        let _: () = e.invoke_contract(
            &router,
            &Symbol::new(&e, "batch_redeem_liquidity"),
            args,
        );

        e.events().publish(
            (symbol_short!("acct"), symbol_short!("bred")),
            plan_count,
        );
        Ok(())
    }

    /* ---- swaps ---- */
    pub fn swap(
        e: Env,
        token_in: Address,
        token_out: Address,
        amount: i128,
        best_hop: Address,
        deadline: u32,
    ) -> Result<(), AccountError> {
        let owner = Self::owner(&e);
        owner.require_auth();

        let router = Self::router(&e);

        // Transfer tokens to the router (Account is direct caller → auth works)
        // This avoids the auth chain issue where external AMMs call
        // require_auth() on the Account address deep in the call stack.
        TokenClient::new(&e, &token_in)
            .transfer(&e.current_contract_address(), &router, &amount);

        // Execute swap via router - tokens are now in the router
        let router_client = RouterClient::new(&e, &router);
        router_client.swap(&amount, &token_in, &token_out, &best_hop, &e.current_contract_address(), &(deadline as u64));

        e.events().publish(("acct", symbol_short!("swap")),
            TokenEvent{ token: token_in, amount });
        Ok(())
    }

    /* ---- passkey lifecycle ---- */

    /// Initialize a smart account with a passkey as primary signer.
    /// For passkey accounts, the `owner` field is set but all auth routes through
    /// `__check_auth` (secp256r1 verification) once `PasskeyPubkey` is stored.
    pub fn initialize_with_passkey(
        e: Env,
        owner: Address,
        router: Address,
        passkey_pubkey: BytesN<65>,
    ) -> Result<(), AccountError> {
        if e.storage().instance().has(&Key::Owner) {
            return Err(AccountError::AlreadyInitialized);
        }
        e.storage().instance().set(&Key::Owner, &owner);
        e.storage().instance().set(&Key::Router, &router);
        e.storage().instance().set(&Key::PasskeyPubkey, &passkey_pubkey);
        Ok(())
    }

    /// Store or replace the passkey public key (65-byte uncompressed secp256r1).
    /// If a passkey is already set, the caller must authenticate via the existing
    /// passkey (routed through `__check_auth`).
    pub fn set_passkey_pubkey(e: Env, pubkey: BytesN<65>) {
        if e.storage().instance().has(&Key::PasskeyPubkey) {
            e.current_contract_address().require_auth();
        }
        e.storage().instance().set(&Key::PasskeyPubkey, &pubkey);
    }

    /// Returns the stored passkey public key, or None if not set.
    pub fn get_passkey_pubkey(e: Env) -> Option<BytesN<65>> {
        e.storage().instance().get(&Key::PasskeyPubkey)
    }

    /* ---- session key lifecycle ---- */

    /// Create a time-limited session key. Requires passkey auth (owner.require_auth()).
    /// The session key allows signing transactions with a temporary ed25519 keypair
    /// instead of requiring a biometric prompt for each TX.
    pub fn create_session(e: Env, session_key: SessionKey) -> Result<(), AccountError> {
        Self::owner(&e).require_auth();
        let ttl = session_key.expiry_ledger.saturating_sub(e.ledger().sequence());
        e.storage().temporary().set(&Key::SessionKey, &session_key);
        e.storage().temporary().extend_ttl(&Key::SessionKey, ttl, ttl);
        e.events().publish(
            (symbol_short!("acct"), symbol_short!("sess")),
            session_key.expiry_ledger,
        );
        Ok(())
    }

    /// Revoke an active session key. Requires passkey auth.
    pub fn revoke_session(e: Env) -> Result<(), AccountError> {
        Self::owner(&e).require_auth();
        e.storage().temporary().remove(&Key::SessionKey);
        Ok(())
    }

    /// Check if a session key is active.
    pub fn has_session(e: Env) -> bool {
        e.storage().temporary().has(&Key::SessionKey)
    }

    /* ---- views ---- */
    pub fn owner(e: &Env)  -> Address { e.storage().instance().get(&Key::Owner).unwrap() }
    pub fn router(e: &Env) -> Address { e.storage().instance().get(&Key::Router).unwrap() }
}

/// WebAuthn passkey authentication via secp256r1, or ed25519 session key.
/// Once `CustomAccountInterface` is implemented, ALL `require_auth()` calls on
/// this contract route through `__check_auth`. This means passkey accounts verify
/// signatures on-chain using the stored secp256r1 public key, or a temporary
/// ed25519 session key for delegated signing.
#[contractimpl]
impl CustomAccountInterface for Account {
    type Error = AccountError;
    type Signature = AccountSignature;

    #[allow(non_snake_case)]
    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signature: AccountSignature,
        auth_contexts: Vec<Context>,
    ) -> Result<(), AccountError> {
        match signature {
            AccountSignature::Passkey(sig) => {
                let pk: BytesN<65> = env
                    .storage()
                    .instance()
                    .get(&Key::PasskeyPubkey)
                    .ok_or(AccountError::PasskeyNotSet)?;
                verify::verify_secp256r1_signature(&env, &signature_payload, &pk, sig);
            }
            AccountSignature::Session(sig) => {
                let session: SessionKey = env
                    .storage()
                    .temporary()
                    .get(&Key::SessionKey)
                    .ok_or(AccountError::NoActiveSession)?;

                // Check expiry
                if env.ledger().sequence() > session.expiry_ledger {
                    return Err(AccountError::SessionExpired);
                }

                // Check scope — every invoked function must be in the allowed list
                for ctx in auth_contexts.iter() {
                    if let Context::Contract(c) = ctx {
                        if c.contract == env.current_contract_address() {
                            let mut allowed = false;
                            for s in session.scope.iter() {
                                if s == c.fn_name {
                                    allowed = true;
                                    break;
                                }
                            }
                            if !allowed {
                                return Err(AccountError::SessionScopeViolation);
                            }
                        }
                    }
                }

                // Verify ed25519 signature
                env.crypto().ed25519_verify(
                    &session.pubkey,
                    &signature_payload.into(),
                    &sig,
                );
            }
        }
        Ok(())
    }
}
