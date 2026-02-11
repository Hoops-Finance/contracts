#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, contracterror,
    token::Client as TokenClient, Address, BytesN, Env, Vec,
};

pub mod hoops_router {
    soroban_sdk::contractimport!(
        file = "../bytecodes/hoops_router.wasm"
    );
    pub type RouterClient<'a> = Client<'a>;
}
use hoops_router::{LpPlan, RouterClient};


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

    /* ---- liquidity flow  (one token in, LP out) ---- */
    pub fn deposit(
        e: Env,
        usdc: Address,
        amount: i128,
        lp_plans: Vec<LpPlan>,
        deadline: u32,
    ) -> Result<(), AccountError> {
        let owner = Self::owner(&e);
        owner.require_auth();
        let tk = TokenClient::new(&e,&usdc);
        tk.transfer(&owner, &e.current_contract_address(), &amount);
        tk.approve(&e.current_contract_address(), &Self::router(&e), &amount, &deadline);
        
        let router_client = RouterClient::new(&e, &Self::router(&e));
        router_client.provide_liquidity(&amount, &lp_plans, &e.current_contract_address(), &(deadline as u64));
        
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
        TokenClient::new(&e,&lp_token)
            .approve(&e.current_contract_address(), &Self::router(&e), &lp_amount, &deadline);
        
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

        // Approve token_in to router
        TokenClient::new(&e, &token_in)
            .approve(&e.current_contract_address(), &Self::router(&e), &amount, &deadline);

        // Execute swap via router
        let router_client = RouterClient::new(&e, &Self::router(&e));
        router_client.swap(&amount, &token_in, &token_out, &best_hop, &e.current_contract_address(), &(deadline as u64));

        e.events().publish(("acct", symbol_short!("swap")),
            TokenEvent{ token: token_in, amount });
        Ok(())
    }

    /* ---- views ---- */
    pub fn owner(e: &Env)  -> Address { e.storage().instance().get(&Key::Owner).unwrap() }
    pub fn router(e: &Env) -> Address { e.storage().instance().get(&Key::Router).unwrap() }
}
