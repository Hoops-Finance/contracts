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
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Vec, I256,unwrap::UnwrapOptimized};

const PROTOCOL_ID: i128 = 1;
pub const STROOP: i128 = 10i128.pow(7);
pub const STROOP_SCALAR: i128 = 10i128.pow(11);

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
        // --- Calculate pool_amount_out using real pool metadata ---
        let pool_total = pool.get_total_supply();
        let swap_fee = pool.get_swap_fee();
        // Fetch real balance and weight for token_a
        let balance_a = pool.get_balance(&token_a.clone());
        let weight_a = pool.get_normalized_weight(&token_a.clone());
        // For now, assume scalar is 10_000_000 (7 decimals)
        let scalar_a = 10_000_000;
        let record_a = Record {
            balance: balance_a,
            weight: weight_a,
            scalar: scalar_a,
            index: 0,
        };
        let pool_amount_out = calc_lp_token_amount_given_token_deposits_in(
            &e,
            &record_a,
            pool_total,
            amt_a,
            swap_fee,
        );
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
        let before_a = token::Client::new(&e, &tokens.get_unchecked(0)).balance(&to);
        let before_b = token::Client::new(&e, &tokens.get_unchecked(1)).balance(&to);
        let before_lp = pool.balance(&to);
        pool.exit_pool(
            &lp_amount,
            &min_amounts_out,
            &to
        );
        bump(&e);
        let after_a = token::Client::new(&e, &tokens.get_unchecked(0)).balance(&to);
        let after_b = token::Client::new(&e, &tokens.get_unchecked(1)).balance(&to);
        let after_lp = pool.balance(&to);
        let actual_a = after_a - before_a;
        let actual_b = after_b - before_b;
        // Optionally, check that LP tokens decreased by lp_amount
        let _lp_diff = before_lp - after_lp;
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

const BONE: i128 = 10i128.pow(18);

fn upscale(e: &Env, amount: i128, scalar: i128) -> I256 {
    I256::from_i128(e, amount * scalar)
}
fn downscale_floor(e: &Env, amount: &I256, scalar: i128) -> i128 {
    let scale_256 = I256::from_i128(e, scalar);
    let one = I256::from_i32(e, 1);
    let result = amount.fixed_div_floor(&e, &scale_256, &one).to_i128();
    result.unwrap_optimized()
}
fn sub_no_negative(e: &Env, a: &I256, b: &I256) -> I256 {
    assert!(a >= b, "sub_no_negative underflow");
    a.sub(&b)
}
fn c_pow(e: &Env, base: &I256, exp: &I256, round_up: bool) -> I256 {
    let bone = I256::from_i128(e, BONE);
    let int = exp.div(&bone);
    let remain = exp.sub(&int.mul(&bone));
    let whole_pow = c_powi(e, &base, &(int.to_i128().unwrap_optimized() as u32));
    if remain == I256::from_i128(e, 0) {
        return whole_pow;
    }
    let partial_result = c_pow_approx(
        e,
        &base,
        &remain,
        &I256::from_i128(e, 1_000_000_000_000_000_000),
        round_up,
    );
    if round_up {
        whole_pow.fixed_mul_ceil(e, &partial_result, &bone)
    } else {
        whole_pow.fixed_mul_floor(e, &partial_result, &bone)
    }
}
fn c_powi(e: &Env, a: &I256, n: &u32) -> I256 {
    let bone = I256::from_i128(e, BONE);
    let mut z = if n % 2 != 0 { a.clone() } else { bone.clone() };
    let mut a = a.clone();
    let mut n = *n / 2;
    while n != 0 {
        a = a.fixed_mul_floor(e, &a, &bone);
        if n % 2 != 0 {
            z = z.fixed_mul_floor(e, &a, &bone);
        }
        n = n / 2
    }
    z
}
fn c_pow_approx(e: &Env, base: &I256, exp: &I256, precision: &I256, round_up: bool) -> I256 {
    let bone = I256::from_i128(e, BONE);
    let zero = I256::from_i32(e, 0);
    let n_1 = I256::from_i32(e, -1);
    let x = base.sub(&bone);
    let mut term = bone.clone();
    let mut sum = term.clone();
    let prec = precision.clone();
    for i in 1..51 {
        let big_k = I256::from_i128(e, i * BONE);
        let c = exp.sub(&big_k.sub(&bone));
        term = term.fixed_mul_floor(e, &c.fixed_mul_floor(e, &x, &bone), &bone);
        term = term.fixed_div_floor(e, &big_k, &bone);
        sum = sum.add(&term);
        let abs_term = if term < zero { term.mul(&n_1) } else { term.clone() };
        if abs_term <= prec { break; }
    }
    if x > zero {
        if term > zero && !round_up {
            sum = sum.sub(&term);
        } else if term < zero && round_up {
            sum = sum.sub(&term);
        }
    } else if !round_up {
        sum = sum.add(&term);
    }
    sum
}
fn calc_lp_token_amount_given_token_deposits_in(
    e: &Env,
    record: &Record,
    pool_supply: i128,
    token_amount_in: i128,
    swap_fee: i128,
) -> i128 {
    let bone = I256::from_i128(e, BONE);
    let token_balance_in = upscale(e, record.balance, record.scalar);
    let token_amount_in = upscale(e, token_amount_in, record.scalar);
    let pool_supply = upscale(e, pool_supply, STROOP_SCALAR);
    let fee = upscale(e, swap_fee, STROOP_SCALAR);
    let normalized_weight = upscale(e, record.weight, STROOP_SCALAR);
    let zaz = bone.sub(&normalized_weight).fixed_mul_floor(e, &fee, &bone);
    let token_amount_in_after_fee = token_amount_in.fixed_mul_floor(&e, &bone.sub(&zaz), &bone);
    let new_token_balance_in = token_balance_in.add(&token_amount_in_after_fee);
    let balance_ratio = new_token_balance_in.fixed_div_floor(&e, &token_balance_in, &bone);
    let pool_ratio = c_pow(e, &balance_ratio, &normalized_weight, false);
    let new_pool_supply = pool_ratio.fixed_mul_floor(&e, &pool_supply, &bone);
    downscale_floor(
        e,
        &sub_no_negative(e, &new_pool_supply, &pool_supply),
        STROOP_SCALAR,
    )
}
