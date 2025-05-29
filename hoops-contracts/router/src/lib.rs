#![no_std]

pub mod client;
pub use client::RouterTrait;

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Vec, contracterror};
use hoops_adapter_interface::AdapterClient;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RouterError {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    AdapterMissing = 3,
    AdapterFailed = 4,
}

#[contracttype]
#[derive(Clone)]
enum Key { Admin, AdapterList }                       // Vec<(i128,Address)>

#[contracttype]
#[derive(Clone)]
pub struct LpPlan {
    pub adapter_id: i128,
    pub token_a:    Address,
    pub token_b:    Address,
    pub proportion: u32,   // simple weight
}

#[contract]
pub struct Router;

#[contractimpl]
impl client::RouterTrait for Router {
    /* ---------- lifecycle ---------- */
    fn initialize(e: Env, admin_addr: Address) -> Result<(), RouterError> {
        if e.storage().instance().has(&Key::Admin) {
            return Err(RouterError::AlreadyInitialized);
        }
        admin_addr.require_auth();
        e.storage().instance().set(&Key::Admin, &admin_addr);
        e.storage().instance().set(&Key::AdapterList, &Vec::<(i128,Address)>::new(&e));
        e.events().publish(("router","init"), admin_addr);
        Ok(())
    }
    
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), RouterError> {
        Self::admin(&e).require_auth();
        e.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }

    /* ---------- admin ops ---------- */
    fn add_adapter(e: Env, id: i128, adapter: Address) -> Result<(), RouterError> {
        Self::admin(&e).require_auth();
        let list: Vec<(i128,Address)> =
            e.storage().instance().get(&Key::AdapterList).unwrap_or(Vec::new(&e));
        // Remove existing adapter with same id if any
        let mut new_list = Vec::new(&e);
        for item in list.iter() {
            if item.0 != id {
                new_list.push_back(item);
            }
        }
        new_list.push_back((id, adapter));
        e.storage().instance().set(&Key::AdapterList, &new_list);
        Ok(())
    }

    fn remove_adapter(e: Env, id: i128) -> Result<(), RouterError> {
        Self::admin(&e).require_auth();
        let list: Vec<(i128,Address)> =
            e.storage().instance().get(&Key::AdapterList).unwrap_or(Vec::new(&e));
        let mut new_list = Vec::new(&e);
        for item in list.iter() {
            if item.0 != id {
                new_list.push_back(item);
            }
        }
        e.storage().instance().set(&Key::AdapterList, &new_list);
        Ok(())
    }

    /* ---------- swaps ---------- */
    fn swap_exact_in(
        e: Env,
        adapter_id: i128,
        amount_in: i128,
        min_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<i128, RouterError> {
        let adapter = Self::adapter_addr(&e, adapter_id)?;
        match AdapterClient::new(&e, &adapter).try_swap_exact_in(
            &amount_in, &min_out, &path, &to, &deadline) {
            Ok(amount_out) => Ok(amount_out.unwrap()),
            Err(_) => Err(RouterError::AdapterFailed)
        }
    }

    /* ---------- liquidity (stubs) ---------- */
    fn provide_liquidity(
        _e: Env, _usdc_amt: i128, _plans: Vec<LpPlan>,
        _beneficiary: Address, _deadline: u64,
    ) -> Result<(), RouterError> { 
        todo!("split, swap, add_liquidity, mint HLPT") 
    }

    fn redeem_liquidity(
        _e: Env, _lp_token: Address, _lp_amt: i128,
        _beneficiary: Address, _deadline: u64,
    ) -> Result<(), RouterError> { 
        todo!() 
    }

    fn admin(e: &Env) -> Address {
        e.storage().instance().get(&Key::Admin).unwrap()
    }
}

impl Router {
    /* ---------- helpers ---------- */
    fn adapter_addr(e: &Env, id: i128) -> Result<Address, RouterError> {
        let list: Vec<(i128,Address)> = e.storage().instance().get(&Key::AdapterList).unwrap_or(Vec::new(e));
        for item in list.iter() {
            if item.0 == id {
                return Ok(item.1);
            }
        }
        Err(RouterError::AdapterMissing)
    }
}
