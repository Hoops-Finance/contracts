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

pub trait CometAdapterTrait {
    /// Register a pool for a set of tokens (sorted for canonicalization)
    fn set_pool_for_tokens(e: Env, tokens: Vec<Address>, pool: Address);
    /// Get a pool for a set of tokens (sorted for canonicalization)
    fn get_pool_for_tokens(e: Env, tokens: Vec<Address>) -> Option<Address>;
}
#[contractimpl]
impl CometAdapterTrait for CometAdapter {
    /// Register a pool for a set of tokens (sorted for canonicalization)
    fn set_pool_for_tokens(e: Env, tokens: Vec<Address>, pool: Address) {
        set_pool_for_tokens(&e, &tokens, &pool);
    }
    /// Get a pool for a set of tokens (sorted for canonicalization)
    fn get_pool_for_tokens(e: Env, tokens: Vec<Address>) -> Option<Address> {
        get_pool_for_tokens(&e, &tokens)
    }
}

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
        to.require_auth();
        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        // Use pool mapping
        let pool_addr = CometAdapter::get_pool_for_tokens(e.clone(), path.clone())
            .ok_or(AdapterError::ExternalFailure)?;
        let pool = CometPoolClient::new(&e, &pool_addr);
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
        to.require_auth();
        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        // Use pool mapping
        let pool_addr = CometAdapter::get_pool_for_tokens(e.clone(), path.clone())
            .ok_or(AdapterError::ExternalFailure)?;
        let pool = CometPoolClient::new(&e, &pool_addr);
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
        bump(&e);
        Ok(amt_in)
    }

    /* ---------- liquidity ---------- */
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
        to.require_auth();
        if !is_init(&e) { return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        let tokens = Vec::from_array(&e, [token_a.clone(), token_b.clone()]);
        let pool_addr = CometAdapter::get_pool_for_tokens(e.clone(), tokens.clone())
            .ok_or(AdapterError::ExternalFailure)?;
        let pool = CometPoolClient::new(&e, &pool_addr);
        let max_amounts = Vec::from_array(&e, [amt_a, amt_b]);
        let pool_amount_out = amt_a.min(amt_b);
        pool.join_pool(
            &pool_amount_out,
            &max_amounts,
            &to
        );
        bump(&e);
        Ok((amt_a, amt_b, pool_amount_out))
    }

    fn remove_liquidity(
        e: Env,
        lp_token: Address,
        lp_amount: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64
    ) -> Result<(i128, i128), AdapterError> {
        #[allow(unused_variables)]
        {
            to.require_auth();
            if !is_init(&e) { return Err(AdapterError::ExternalFailure); }
            if e.ledger().timestamp() > deadline {
                return Err(AdapterError::ExternalFailure);
            }
            // Use pool mapping
            // For two-token pools, use the LP token and the two underlying tokens
            // Here, we assume the LP token is unique per pool, so we use it as the key
            // If needed, update to use the correct token set
            let pool_addr = CometAdapter::get_pool_for_tokens(e.clone(), Vec::from_array(&e, [lp_token.clone(), lp_token.clone()]))
                .ok_or(AdapterError::ExternalFailure)?;
            let pool = CometPoolClient::new(&e, &pool_addr);
            let min_amounts_out = Vec::from_array(&e, [amt_a_min, amt_b_min]);
            pool.exit_pool(
                &lp_amount,
                &min_amounts_out,
                &to
            );
            bump(&e);
            Ok((amt_a_min, amt_b_min))
        }
    }
}
