#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, contracterror,
    token::Client as TokenClient, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

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


#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccountError {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
}

#[contracttype]
#[derive(Clone)]
enum Key { Owner, Router }

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

    /* ---- views ---- */
    pub fn owner(e: &Env)  -> Address { e.storage().instance().get(&Key::Owner).unwrap() }
    pub fn router(e: &Env) -> Address { e.storage().instance().get(&Key::Router).unwrap() }
}
