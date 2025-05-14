#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token::Client as TokenClient, Address, BytesN, Env, Vec,
};
use hoops_common::{admin, CommonError};

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
    pub fn initialize(e: Env, owner: Address, router: Address) -> Result<(), CommonError> {
        if e.storage().instance().has(&Key::Owner) {
            return Err(CommonError::AlreadyInitialised)
        }
        owner.require_auth();
        e.storage().instance().set(&Key::Owner, &owner);
        e.storage().instance().set(&Key::Router, &router);
        admin::set(&e, owner);
        Ok(())
    }
    pub fn upgrade(e: Env, wasm: [u8;32]) -> Result<(), CommonError> {
        admin::upgrade(&e, BytesN::from_array(&e,&wasm))
    }

    /* ---- token passthrough ---- */
    pub fn transfer(e: Env, token: Address, to: Address, amount: i128) -> Result<(), CommonError> {
        Self::owner(&e).require_auth();
        TokenClient::new(&e,&token)
            .transfer(&e.current_contract_address(), &to, &amount);
        e.events().publish(("acct", symbol_short!("xfer")),
                           TokenEvent{ token, amount });
        Ok(())
    }

    /* ---- liquidity flow (USDC in, LP out) ---- */
    pub fn deposit_usdc(
        e: Env,
        usdc: Address,
        amount: i128,
        lp_plans: Vec<BytesN<32>>,
        deadline: u64,
    ) -> Result<(), CommonError> {
        let owner = Self::owner(&e);
        owner.require_auth();
        let tk = TokenClient::new(&e,&usdc);
        tk.transfer(&owner, &e.current_contract_address(), &amount);
        tk.increase_allowance(&e.current_contract_address(), &Self::router(&e), &amount);
        e.invoke_contract(&Self::router(&e), &"provide_liquidity",
            (amount, lp_plans, e.current_contract_address(), deadline));
        e.events().publish(("acct", symbol_short!("dep")),
            TokenEvent{ token: usdc, amount });
        Ok(())
    }

    pub fn redeem(
        e: Env,
        lp_token: Address,
        lp_amount: i128,
        usdc: Address,
        deadline: u64,
    ) -> Result<(), CommonError> {
        Self::owner(&e).require_auth();
        TokenClient::new(&e,&lp_token)
            .increase_allowance(&e.current_contract_address(), &Self::router(&e), &lp_amount);
        e.invoke_contract(&Self::router(&e), &"redeem_liquidity",
            (lp_token, lp_amount, e.current_contract_address(), deadline));
        // sweep USDC to owner
        let bal = TokenClient::new(&e,&usdc).balance(&e.current_contract_address());
        TokenClient::new(&e,&usdc)
            .transfer(&e.current_contract_address(), &Self::owner(&e), &bal);
        e.events().publish(("acct", symbol_short!("wd")),
            TokenEvent{ token: usdc, amount: bal });
        Ok(())
    }

    /* ---- views ---- */
    pub fn owner(e: &Env)  -> Address { e.storage().instance().get_unchecked(&Key::Owner) }
    pub fn router(e: &Env) -> Address { e.storage().instance().get_unchecked(&Key::Router) }
}
