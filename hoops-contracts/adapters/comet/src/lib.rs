#![no_std]

mod storage;
mod event;
mod protocol;

use storage::*;
#[allow(unused_imports)]
use event::*;
use protocol::CometPoolClient;
use hoops_adapter_interface::{AdapterTrait, AdapterError};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

const PROTOCOL_ID: i128 = 1;

#[contract]
pub struct CometAdapter;

#[contractimpl]
impl AdapterTrait for CometAdapter {

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

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), AdapterError> {
        // setup the config to get the admin address. set it during initialization.
        let config = get_core_config(&e);
        config.admin.require_auth();
        Ok(e.deployer().update_current_contract_wasm(new_wasm_hash))
    }

    /* ---------- swaps ---------- */
    fn swap_exact_in(
        e: Env,
        amount_in: i128,
        min_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64
    ) -> Result<i128, AdapterError> {

        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }

        let pool = CometPoolClient::new(&e, &get_amm(&e)?);
        // Only support single-hop for now
        let offer_asset = path.get(0).ok_or(AdapterError::UnsupportedPair)?;
        let ask_asset = path.get(1).ok_or(AdapterError::UnsupportedPair)?;
        
        let (amt_out, _) = pool.swap_exact_amount_in(
            &offer_asset,
            &amount_in,
            &ask_asset,
            &min_out,
            &i128::MAX, // max_price
            &to
        );
// THIS DOES NOT I'm not sure what you're trying to do but it needs fixed.
//.map_err(|_| AdapterError::ExternalFailure)?;
        
        bump(&e);
        Ok(amt_out)
    }

    fn swap_exact_out(
        e: Env, 
        amount_out: i128, 
        max_in: i128, 
        path: Vec<Address>,
        to: Address, 
        deadline: u64
    ) -> Result<i128, AdapterError> {
        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        
        let pool = CometPoolClient::new(&e, &get_amm(&e)?);
        // Only support single-hop for now
        let offer_asset = path.get(0).ok_or(AdapterError::UnsupportedPair)?;
        let ask_asset = path.get(1).ok_or(AdapterError::UnsupportedPair)?;
        
        let (amt_in, _) = pool.swap_exact_amount_out(
            &offer_asset,
            &max_in,
            &ask_asset,
            &amount_out,
            &i128::MAX, // max_price
            &to
        );
// THIS DOES NOT I'm not sure what you're trying to do but it needs fixed.
//.map_err(|_| AdapterError::ExternalFailure)?;
        
        bump(&e);
        Ok(amt_in)
    }

    /* ---------- liquidity ---------- */
    #[allow(unused_variables)]
    fn add_liquidity(
        e: Env,
        token_a: Address,
        token_b: Address,
        amt_a: i128,
        amt_b: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64
    ) -> Result<(i128, i128, i128), AdapterError> {
        if !is_init(&e) { return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }

        let pool = CometPoolClient::new(&e, &get_amm(&e)?);
        // Use min amounts for slippage protection
        let max_amounts = Vec::from_array(&e, [amt_a, amt_b]);
        let min_amounts = Vec::from_array(&e, [amt_a_min, amt_b_min]);
        // Calculate LP tokens to mint (simplified, should be based on pool state)
        let pool_amount_out = amt_a.min(amt_b); // Placeholder logic

        // In a real implementation, join_pool should use min_amounts for slippage checks
        pool.join_pool(
            &pool_amount_out,
            &max_amounts,
            &to
        );

        bump(&e);
        Ok((amt_a, amt_b, pool_amount_out))
    }
#[allow(unused_variables)]
    fn remove_liquidity(
        e: Env,
        lp_token: Address,
        lp_amount: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64
    ) -> Result<(i128, i128), AdapterError> {
        if !is_init(&e) { return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }

        let pool = CometPoolClient::new(&e, &get_amm(&e)?);
        // Use min amounts for slippage protection
        let min_amounts_out = Vec::from_array(&e, [amt_a_min, amt_b_min]);

        pool.exit_pool(
            &lp_amount,
            &min_amounts_out,
            &to
        );

        // In a real implementation, should return actual withdrawn amounts
        let amt_a = lp_amount / 2; // Placeholder
        let amt_b = lp_amount / 2;

        bump(&e);
        Ok((amt_a, amt_b))
    }
}
