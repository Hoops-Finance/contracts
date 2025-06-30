#![no_std]

mod storage;
mod event;
mod protocol;

use soroban_fixed_point_math::{FixedPoint, SorobanFixedPoint};
use storage::*;
#[allow(unused_imports)]
use event::*;
use protocol::CometPoolClient;
use hoops_adapter_interface::{AdapterTrait, AdapterError};
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Vec, I256, unwrap::UnwrapOptimized};

const PROTOCOL_ID: i128 = 1;
pub const STROOP: i128 = 10i128.pow(7);
pub const STROOP_SCALAR: i128 = 10i128.pow(11);
/// Fixed-point scale expected by c_math helpers (1 e-9).
pub const ONE_E9: i128 = 1_000_000_000;

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
        if is_init(&e) { return Err(AdapterError::AlreadyInitialized); }
        if amm_id != PROTOCOL_ID { return Err(AdapterError::InvalidID); }

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
        _amt_a_min: i128,
        _amt_b_min: i128,
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

        // Fetch pool metadata
        let pool_total = pool.get_total_supply();
        let balances = Vec::from_array(&e, [pool.get_balance(&token_a), pool.get_balance(&token_b)]);

        let decimals_a = token::Client::new(&e, &token_a).decimals();
        let decimals_b = token::Client::new(&e, &token_b).decimals();
        let scalars = Vec::from_array(&e, [10i128.pow(decimals_a as u32), 10i128.pow(decimals_b as u32)]);

        // Calculate join ratios for each token
        let mut min_ratio = None;
        for i in 0..2 {
            let user_amt = max_amounts.get_unchecked(i);
            let pool_bal = balances.get_unchecked(i);
            let scalar = scalars.get_unchecked(i);
            // Upscale user_amt and pool_bal to I256
            let user_amt_scaled = I256::from_i128(&e, user_amt * scalar);
            let pool_bal_scaled = I256::from_i128(&e, pool_bal * scalar);
            // ratio = user_amt / pool_bal
            let ratio = if pool_bal_scaled > I256::from_i32(&e, 0) {
                user_amt_scaled.fixed_div_floor(&e, &pool_bal_scaled, &I256::from_i128(&e, ONE_E9))
            } else {
                I256::from_i32(&e, 0)
            };
            min_ratio = Some(match min_ratio {
                None => ratio,
                Some(r) => if ratio < r { ratio } else { r },
            });
        }
        // pool_amount_out = pool_total * min_ratio
        let pool_total_256 = I256::from_i128(&e, pool_total);
        let min_ratio = min_ratio.unwrap_or(I256::from_i32(&e, 0));
        let pool_amount_out_256 = pool_total_256.fixed_mul_floor(&e, &min_ratio, &I256::from_i128(&e, ONE_E9));
        let pool_amount_out = pool_amount_out_256.to_i128().unwrap_optimized();

        let before_lp = pool.balance(&to);
        let before_a = token::Client::new(&e, &token_a).balance(&to);
        let before_b = token::Client::new(&e, &token_b).balance(&to);
        pool.join_pool(
            &pool_amount_out,
            &max_amounts,
            &to
        );
        bump(&e);
        let after_lp = pool.balance(&to);
        let after_a = token::Client::new(&e, &token_a).balance(&to);
        let after_b = token::Client::new(&e, &token_b).balance(&to);
        let actual_lp = after_lp - before_lp;
        let actual_a = before_a - after_a;
        let actual_b = before_b - after_b;
        Ok((actual_a, actual_b, actual_lp))
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
        to.require_auth();
        if !is_init(&e) { return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        // For Comet, the LP token address is the pool address
        let pool = CometPoolClient::new(&e, &lp_token);
        let min_amounts_out = Vec::from_array(&e, [amt_a_min, amt_b_min]);
        let tokens = pool.get_tokens();
        let token_a = tokens.get_unchecked(0);
        let token_b = tokens.get_unchecked(1);
        
        let before_a = token::Client::new(&e, &token_a).balance(&to);
        let before_b = token::Client::new(&e, &token_b).balance(&to);
       
        pool.exit_pool(
            &lp_amount,
            &min_amounts_out,
            &to
        );
        bump(&e);
        let after_a = token::Client::new(&e, &token_a).balance(&to);
        let after_b = token::Client::new(&e, &token_b).balance(&to);
        let actual_a = after_a - before_a;
        let actual_b = after_b - before_b;
       
        Ok((actual_a, actual_b))
    }
}

// --- Comet math and Record struct (copied for adapter self-containment) ---
#[derive(Clone, Debug)]
pub struct Record {
    pub balance: i128,
    pub weight: i128,
    pub scalar: i128,
    pub index: u32,
}
