#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Vec};
use hoops_common::{admin, CommonError, CommonEvent};
use hoops_adapter_interface::{AdapterClient, AdapterError};

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RouterError {
    Common(CommonError as u32) = 300,
    AdapterMissing             = 301,
    AdapterFailed              = 302,
}

#[derive(Clone)]
enum Key { AdapterList }                       // Vec<(i128,Address)>

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
impl Router {
    /* ---------- lifecycle ---------- */
    pub fn initialize(e: Env, admin_addr: Address) {
        admin_addr.require_auth();
        admin::set(&e, admin_addr.clone());
        e.storage().instance().set(&Key::AdapterList, &Vec::<(i128,Address)>::new(&e));
        e.events().publish(("router","init"), CommonEvent::Init{ admin: admin_addr });
    }
    pub fn upgrade(e: Env, hash: [u8;32]) -> Result<(), RouterError> {
        admin::upgrade(&e, BytesN::from_array(&e,&hash))
             .map_err(|e| RouterError::Common(e as u32))
    }

    /* ---------- admin ops ---------- */
    pub fn set_adapter(e: Env, id: i128, addr: Address) -> Result<(), RouterError> {
        admin::require(&e).map_err(|e| RouterError::Common(e as u32))?;
        let mut list: Vec<(i128,Address)> =
            e.storage().instance().get(&Key::AdapterList).unwrap_or(Vec::new(&e));
        list.retain(|(old,_)| *old != id);
        list.push_back((id,addr));
        e.storage().instance().set(&Key::AdapterList, &list);
        Ok(())
    }

    /* ---------- helpers ---------- */
    fn adapter_addr(e: &Env, id: i128) -> Result<Address, RouterError> {
        let list: Vec<(i128,Address)> = e.storage().instance().get_unchecked(&Key::AdapterList);
        list.iter().find(|(k,_)| *k==id)
            .map(|(_,a)| a.clone())
            .ok_or(RouterError::AdapterMissing)
    }

    /* ---------- swaps ---------- */
    pub fn swap_exact_in(
        e: Env,
        adapter_id: i128,
        amount_in: i128,
        min_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<i128, RouterError> {
        let adapter = Self::adapter_addr(&e, adapter_id)?;
        AdapterClient::new(&e,&adapter).swap_exact_in(
            &amount_in,&min_out,&path,&to,&deadline)
            .map_err(|_| RouterError::AdapterFailed)
    }

    /* ---------- liquidity (stubs) ---------- */
    pub fn provide_liquidity(
        _e: Env, _usdc_amt: i128, _plans: Vec<LpPlan>,
        _beneficiary: Address, _deadline: u64,
    ) { todo!("split, swap, add_liquidity, mint HLPT") }

    pub fn redeem_liquidity(
        _e: Env, _lp_token: Address, _lp_amt: i128,
        _beneficiary: Address, _deadline: u64,
    ) { todo!() }
}
