#![no_std]

mod storage;
mod event;
mod protocol;

use storage::*;
#[allow(unused_imports)]
use event::*;
use hoops_adapter_interface::{AdapterTrait, AdapterError};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};
use protocol::phoenix_pair::PhoenixPoolClient;

const PROTOCOL_ID: i128 = 2;

#[contract]
pub struct PhoenixAdapter;

#[contractimpl]
impl AdapterTrait for PhoenixAdapter {

    fn version() -> u32 {
        1
    }

    /* ---------- lifecycle ---------- */
    fn initialize(e: Env, amm_id: i128, amm_addr: Address) -> Result<(), AdapterError> {
        if is_init(&e) { return Err(AdapterError::ExternalFailure); }
        if amm_id != PROTOCOL_ID { return Err(AdapterError::UnsupportedPair); }

        set_amm(&e, amm_addr.clone());
        mark_init(&e);
        bump(&e);
        init(&e, amm_addr);
        Ok(())
    }

    fn upgrade(_e: Env, _hash: BytesN<32>) -> Result<(), AdapterError> {
        // Stub: always fail for now
        Err(AdapterError::ExternalFailure)
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

        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp()>deadline{
            return Err(AdapterError::ExternalFailure);
        }

        let pool = PhoenixPoolClient::new(&e, &get_amm(&e)?);
        // Only support single-hop for now
        let offer_asset = path.get(0).ok_or(AdapterError::UnsupportedPair)?;
        
        //let ask_asset = path.get(1).ok_or(AdapterError::UnsupportedPair)?;
        let amt_out = pool.swap(
            &to, // sender
            &offer_asset,
            &amt_in,
            &Some(min_out),
            &None, // max_spread_bps
            &Some(deadline as u64),
            &None // max_allowed_fee_bps
        );
        bump(&e);
        Ok(amt_out)
    }

    fn swap_exact_out(
        e: Env, out: i128, max_in: i128, path: Vec<Address>,
        to: Address, deadline: u64
    ) -> Result<i128, AdapterError> {
        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp()>deadline{
            return Err(AdapterError::ExternalFailure); }
        let pool = PhoenixPoolClient::new(&e, &get_amm(&e)?);
        let ask_asset = &path.get(1).ok_or(AdapterError::UnsupportedPair)?;
        let resp = pool.simulate_reverse_swap(&ask_asset, &out);
        let required_in = resp.offer_amount;
        if required_in > max_in {
            return Err(AdapterError::ExternalFailure);
        }
        // Actually perform the swap
        let offer_asset = path.get(0).ok_or(AdapterError::UnsupportedPair)?;
        let _amt_out = pool.swap(
            &to, // sender
            &offer_asset,
            &required_in,
            &Some(out),
            &None, // max_spread_bps
            &Some(deadline as u64),
            &None // max_allowed_fee_bps
        );
        bump(&e);
        Ok(required_in)
    }

    /* ---------- liquidity ---------- */
    fn add_liquidity(
        e: Env, _a: Address, _b: Address, amt_a: i128, amt_b: i128,
        to: Address, deadline: u64
    ) -> Result<(i128,i128,i128), AdapterError> {
        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp()>deadline{
            return Err(AdapterError::ExternalFailure); }
        let pool = PhoenixPoolClient::new(&e, &get_amm(&e)?);
        // For now, require both tokens > 0. One-sided logic can be added later.
        pool.provide_liquidity(
            &to, // sender
            &Some(amt_a),
            &Some(amt_a), // min_a
            &Some(amt_b),
            &Some(amt_b), // min_b
            &None, // custom_slippage_bps
            &Some(deadline),
            &false // auto_stake
        );
        bump(&e);
        // placeholder because we need actual amount_a, amount_b and lp_minted_amount
        Ok((amt_a, amt_b, 0))
    }

    fn remove_liquidity(
        e: Env, _lp: Address, lp_amt: i128, to: Address, deadline: u64
    ) -> Result<(i128,i128), AdapterError> {
        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp()>deadline{
            return Err(AdapterError::ExternalFailure); }

            // why do we need the lp address here? and how are we dealing with auth?
        let pool = PhoenixPoolClient::new(&e, &get_amm(&e)?);
        let (amt_a, amt_b) = pool.withdraw_liquidity(
            &to, // sender
            &lp_amt,
            &0, // min_a
            &0, // min_b
            &Some(deadline),
            &None // auto_unstake
        );
        bump(&e);
        Ok((amt_a, amt_b))
    }
}
