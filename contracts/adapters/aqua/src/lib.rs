#![no_std]

mod storage;
mod event;
mod protocol_interface;

use storage::*;
use event::*;
use protocol_interface::AquaRouterClient;
use hoops_adapter_interface::{AdapterTrait, AdapterError};
use hoops_common::{admin, CommonError};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

const PROTOCOL_ID: i128 = 0;

#[contract]
pub struct AquaAdapter;

#[contractimpl]
impl AdapterTrait for AquaAdapter {

    /* ---------- lifecycle ---------- */
    fn initialize(e: Env, amm_id: i128, amm_addr: Address) -> Result<(), AdapterError> {
        if is_init(&e) { return Err(AdapterError::Common(CommonError::AlreadyInitialised as u32)); }
        if amm_id != PROTOCOL_ID { return Err(AdapterError::UnsupportedPair); }

        admin::set(&e, e.invoker());
        set_amm(&e, amm_addr.clone());
        mark_init(&e);
        bump(&e);
        init(&e, amm_addr);
        Ok(())
    }

    fn upgrade(e: Env, hash: [u8;32]) -> Result<(), AdapterError> {
        admin::upgrade(&e, BytesN::from_array(&e,&hash))
            .map_err(|e| AdapterError::Common(e as u32))
    }

    /* ---------- swaps ---------- */
    fn swap_exact_in(
        e: Env,
        amt_in: i128,
        min_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64
    ) -> Result<i128, AdapterError> {

        if !is_init(&e){ return Err(AdapterError::Common(CommonError::NotInitialised as u32)); }
        if e.ledger().timestamp()>deadline{
            return Err(AdapterError::Common(CommonError::DeadlineExpired as u32));
        }

        // Call external router (replace todo later)
        let _router = AquaRouterClient::new(&e, &get_amm(&e)?);
        // let amt_out = _router.swap_exact_tokens_for_tokens(&amt_in,&min_out,&path,&to,&deadline);
        let amt_out = min_out; // placeholder

        swap(&e, SwapEvent{ amt_in, amt_out, path, to});
        bump(&e);
        Ok(amt_out)
    }

    fn swap_exact_out(
        _e: Env, _out: i128, _max_in: i128, _path: Vec<Address>,
        _to: Address, _deadline: u64
    ) -> Result<i128, AdapterError> {
        todo!("Aqua exact‑out swaps")
    }

    /* ---------- liquidity ---------- */
    fn add_liquidity(
        _e: Env, _a: Address, _b: Address, _amt_a: i128, _amt_b: i128,
        _to: Address, _deadline: u64
    ) -> Result<Address, AdapterError> {
        todo!("Aqua add_liquidity")
    }

    fn remove_liquidity(
        _e: Env, _lp: Address, _lp_amt: i128, _to: Address, _deadline: u64
    ) -> Result<(i128,i128), AdapterError> {
        todo!("Aqua remove_liquidity")
    }
}
