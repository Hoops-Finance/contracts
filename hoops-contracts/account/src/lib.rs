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
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LpPlan {
    pub token_a: Address,
    pub token_b: Address,
    pub amount_a: i128,
    pub amount_b: i128,
    pub adapter_id: i128,
}


#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Secp256r1Signature {
    pub authenticator_data: Bytes,
    pub client_data_json: Bytes,
    pub signature: BytesN<64>,
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
}

#[contracttype]
#[derive(Clone)]
enum Key { Owner, Router, PasskeyPubkey }

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

    pub fn redeem(
        e: Env,
        lp_token: Address,
        lp_amount: i128,
        usdc: Address,
        deadline: u32,
    ) -> Result<(), AccountError> {
        Self::owner(&e).require_auth();
        let approval_ledger = e.ledger().sequence() + 200;
        TokenClient::new(&e,&lp_token)
            .approve(&e.current_contract_address(), &Self::router(&e), &lp_amount, &approval_ledger);

        let router_client = RouterClient::new(&e, &Self::router(&e));
        router_client.redeem_liquidity(&lp_token, &lp_amount, &e.current_contract_address(), &(deadline as u64));

        // sweep USDC to owner
        let bal = TokenClient::new(&e,&usdc).balance(&e.current_contract_address());
        TokenClient::new(&e,&usdc)
            .transfer(&e.current_contract_address(), &Self::owner(&e), &bal);
        e.events().publish(("acct", symbol_short!("wd")),
            TokenEvent{ token: usdc, amount: bal });
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

    /* ---- views ---- */
    pub fn owner(e: &Env)  -> Address { e.storage().instance().get(&Key::Owner).unwrap() }
    pub fn router(e: &Env) -> Address { e.storage().instance().get(&Key::Router).unwrap() }
}

/// WebAuthn passkey authentication via secp256r1.
/// Once `CustomAccountInterface` is implemented, ALL `require_auth()` calls on
/// this contract route through `__check_auth`. This means passkey accounts verify
/// signatures on-chain using the stored secp256r1 public key.
#[contractimpl]
impl CustomAccountInterface for Account {
    type Error = AccountError;
    type Signature = Secp256r1Signature;

    #[allow(non_snake_case)]
    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signature: Secp256r1Signature,
        _auth_contexts: Vec<Context>,
    ) -> Result<(), AccountError> {
        let pk: BytesN<65> = env
            .storage()
            .instance()
            .get(&Key::PasskeyPubkey)
            .ok_or(AccountError::PasskeyNotSet)?;

        verify::verify_secp256r1_signature(&env, &signature_payload, &pk, signature);

        Ok(())
    }
}
