#![no_std]

mod event;
mod protocol;
mod storage;

use event::*;
use hoops_adapter_interface::{AdapterError, AdapterTrait};
use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractimpl, log, panic_with_error, token, token::Client as TokenClient,
    Address, BytesN, Env, IntoVal, Symbol, Vec,
};
use storage::{
    bump, get_amm, get_pool_for_tokens, is_init, mark_init, set_amm, set_pool_for_tokens,
    AquaPoolInfo,
};

use crate::storage::get_core_config;

const PROTOCOL_ID: i128 = 0;
pub fn get_deposit_amounts(
    e: &Env,
    desired_a: u128,
    min_a: u128,
    desired_b: u128,
    min_b: u128,
    reserve_a: u128,
    reserve_b: u128,
) -> (u128, u128) {
    if reserve_a == 0 && reserve_b == 0 {
        return (desired_a, desired_b);
    }

    let amount_b = desired_a.fixed_mul_floor(e, &reserve_b, &reserve_a);
    if amount_b <= desired_b {
        if amount_b < min_b {
            panic_with_error!(e, AdapterError::InvalidAmount);
        }
        (desired_a, amount_b)
    } else {
        let amount_a = desired_b.fixed_mul_floor(&e, &reserve_a, &reserve_b);
        if amount_a > desired_a || desired_a < min_a {
            panic_with_error!(e, AdapterError::InvalidAmount);
        }
        (amount_a, desired_b)
    }
}

pub fn get_shares(
    e: &Env,
    amt_a: u128,
    amt_b: u128,
    reserve_a: u128,
    reserve_b: u128,
    shares: u128,
) -> u128 {
    if reserve_a == 0 || reserve_b == 0 || shares == 0 || amt_a == 0 || amt_b == 0 {
        return 0; // No shares can be created in an empty pool
    }
    let new_reserve_a = reserve_a + amt_a;
    let new_reserve_b = reserve_b + amt_b;
    let shares_a = amt_a.fixed_mul_floor(e, &shares, &new_reserve_a);
    let shares_b = amt_b.fixed_mul_floor(e, &shares, &new_reserve_b);
    shares_a.min(shares_b)
}
#[contract]
pub struct AquaAdapter;

pub trait AquaAdapterTrait {
    fn set_pool_for_tokens(e: Env, tokens: Vec<Address>, info: AquaPoolInfo);
    fn get_pool_for_tokens(e: Env, tokens: Vec<Address>) -> Option<AquaPoolInfo>;
}

#[contractimpl]
impl AquaAdapterTrait for AquaAdapter {
    fn set_pool_for_tokens(e: Env, tokens: Vec<Address>, info: AquaPoolInfo) {
        set_pool_for_tokens(&e, &tokens, &info);
    }
    fn get_pool_for_tokens(e: Env, tokens: Vec<Address>) -> Option<AquaPoolInfo> {
        get_pool_for_tokens(&e, &tokens)
    }
}

#[contractimpl]
impl AdapterTrait for AquaAdapter {
    fn version() -> u32 {
        1
    }
    fn initialize(e: Env, amm_id: i128, amm_addr: Address) -> Result<(), AdapterError> {
        if is_init(&e) {
            return Err(AdapterError::AlreadyInitialized);
        }
        if amm_id != PROTOCOL_ID {
            return Err(AdapterError::InvalidID);
        }

        set_amm(&e, amm_addr.clone());
        mark_init(&e);
        bump(&e);
        init(&e, amm_addr);
        Ok(())
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), AdapterError> {
        let config = get_core_config(&e);
        config.admin.require_auth();
        Ok(e.deployer().update_current_contract_wasm(new_wasm_hash))
    }
    fn swap_exact_in(
        e: Env,
        amt_in: i128,
        min_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<i128, AdapterError> {
        if amt_in < 0 || min_out < 0 {
            return Err(AdapterError::InvalidAmount);
        };
        let amt_in = amt_in as u128;
        let min_out = min_out as u128;
        to.require_auth();
        if !is_init(&e) {
            return Err(AdapterError::ExternalFailure);
        }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        if path.len() != 2 {
            return Err(AdapterError::MultipathUnsupported);
        }
        let pool_info = get_pool_for_tokens(&e, &path).ok_or(AdapterError::UnsupportedPair)?;
        let pool = protocol::AquaPoolClient::new(&e, &pool_info.pool_address);
        let tokens = pool.get_tokens();
        let in_token = path.get(0).unwrap();
        let in_idx = if tokens.get(0).unwrap() == in_token {
            0
        } else {
            1
        };
        let out_idx = if in_idx == 0 { 1 } else { 0 };
        let amt_out = pool.swap(
            &to, // user
            &in_idx,
            &out_idx,
            //todo: convert all our usage of i128 as amounts to u128 for safety.
            &amt_in, 
            &min_out,
        );
        let amt_out_i128 = amt_out as i128;
        event::swap(
            &e,
            event::SwapEvent {
                amt_in: amt_in as i128,
                amt_out: amt_out_i128,
                path,
                to,
            },
        );
        bump(&e);
        Ok(amt_out_i128)
    }

    fn swap_exact_out(
        e: Env,
        out: i128,
        max_in: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<i128, AdapterError> {
        if out < 0 || max_in < 0 {
            return Err(AdapterError::InvalidAmount);
        };
        let out = out as u128;
        let max_in = max_in as u128;
        to.require_auth();
        if !is_init(&e) {
            return Err(AdapterError::ExternalFailure);
        }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        if path.len() != 2 {
            return Err(AdapterError::MultipathUnsupported);
        }
        let pool_info = get_pool_for_tokens(&e, &path).ok_or(AdapterError::UnsupportedPair)?;
        let pool = protocol::AquaPoolClient::new(&e, &pool_info.pool_address);
        let tokens = pool.get_tokens();
        let in_token = path.get(0).unwrap();
        let in_idx = if tokens.get(0).unwrap() == in_token {
            0
        } else {
            1
        };
        let out_idx = if in_idx == 0 { 1 } else { 0 };

        let amt_in = pool.swap_strict_receive(
            &to,
            //todo: convert all our usage of i128 as amounts to u128 for safety.
            &in_idx,
            &out_idx,
            &out,
            &max_in,
        );

        let amt_in_i128 = amt_in as i128;
        event::swap(
            &e,
            event::SwapEvent {
                amt_in: amt_in_i128,
                amt_out: out as i128,
                path,
                to,
            },
        );
        bump(&e);
        Ok(amt_in_i128)
    }
 
    fn add_liquidity(
        e: Env,
        token_a: Address,
        token_b: Address,
        amt_a: i128,
        amt_b: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64,
    ) -> Result<(i128, i128, i128), AdapterError> {
        to.require_auth();
        if amt_a < 0 || amt_b < 0 || amt_a_min < 0 || amt_b_min < 0 {
            return Err(AdapterError::InvalidAmount);
        }
        log!(&e, "testing the logging");
        let amt_a = amt_a as u128;
        let amt_b = amt_b as u128;
        let amt_a_min = amt_a_min as u128;
        let amt_b_min = amt_b_min as u128;
        if amt_a == 0 || amt_b == 0 {
            return Err(AdapterError::InvalidAmount);
        }
        
        if !is_init(&e) {
            return Err(AdapterError::ExternalFailure);
        }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        let tokens = Vec::from_array(&e, [token_a.clone(), token_b.clone()]);
        let pool_info = get_pool_for_tokens(&e, &tokens).ok_or(AdapterError::UnsupportedPair)?;
        let pool = protocol::AquaPoolClient::new(&e, &pool_info.pool_address);
        log!(&e, "Found pool for tokens: {:?}", pool_info.pool_address);

        let reserves = pool.get_reserves();
        log!(&e, "Calling get_reserves: reserves={:?}", reserves);
        let reserve_a = reserves.get(0).unwrap();
        let reserve_b = reserves.get(1).unwrap();
        
        if reserve_a == 0 && reserve_b == 0 {
            return Err(AdapterError::ExternalFailure); // Cannot add liquidity to an empty pool yet.
        }
       let shares = pool.get_total_shares();
       log!(&e, "Calling get_total_shares: shares={}", shares);
        let (amount_a, amount_b) =
            get_deposit_amounts(&e, amt_a, amt_b, amt_a_min, amt_b_min, reserve_a, reserve_b);
        log!(&e, "get_deposit_amounts returned: amount_a={:?}, amount_b={:?}", amount_a, amount_b);
        let desired_amounts = Vec::from_array(&e, [amount_a, amount_b]);
        let min_shares = get_shares(
            &e,
            amount_a,
            amount_b,
            reserve_a,
            reserve_b,
            shares,
        );
        log!(&e, "Calling get_deposit_amounts: amt_a={}, amt_b={}, amt_a_min={}, amt_b_min={}, reserve_a={}, reserve_b={}", amt_a, amt_b, amt_a_min, amt_b_min, reserve_a, reserve_b);
        let (amount_a, amount_b) = get_deposit_amounts(&e, amt_a, amt_b, amt_a_min, amt_b_min, reserve_a, reserve_b);
        log!(&e, "get_deposit_amounts returned: amount_a={}, amount_b={}", amount_a, amount_b);

        log!(&e, "Calling get_shares: amount_a={}, amount_b={}, reserve_a={}, reserve_b={}, shares={}", amount_a, amount_b, reserve_a, reserve_b, shares);
        
        log!(&e, "get_shares returned: min_shares={}", min_shares);
        if min_shares == 0 {
            return Err(AdapterError::InvalidAmount);
        }
        let (amounts, shares) = pool.deposit(&to, &desired_amounts, &min_shares);
        let amount_a_i128 = amounts.get(0).unwrap() as i128;
        let amount_b_i128 = amounts.get(1).unwrap() as i128;
        let shares_i128 = shares as i128;
        bump(&e);
        Ok((amount_a_i128, amount_b_i128, shares_i128))
    }

    fn remove_liquidity(
        e: Env,
        lp: Address,
        lp_amt: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64,
    ) -> Result<(i128, i128), AdapterError> {
        if lp_amt < 0 || amt_a_min < 0 || amt_b_min < 0 {
            return Err(AdapterError::InvalidAmount);
        }
        let lp_amt = lp_amt as u128;
        let amt_a_min = amt_a_min as u128;
        let amt_b_min = amt_b_min as u128;
        to.require_auth();
        if !is_init(&e) {
            return Err(AdapterError::ExternalFailure);
        }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        log!(&e, "Attempting to remove liquidity from Aqua pool with lp token {:?}", lp);
        // Find pool by LP token address
        let pool_info = storage::get_pool_by_lp_token(&e, &lp).ok_or(AdapterError::UnsupportedPair)?;
        log!(&e, "Found pool for LP token: {:?}", pool_info);
        let pool = protocol::AquaPoolClient::new(&e, &pool_info.pool_address);
        let lp_token_client = TokenClient::new(&e, &lp);

        let curr_lp = lp_token_client.balance(&to);
        if curr_lp < lp_amt as i128 {
            return Err(AdapterError::InsufficientBalance);
        }
        let minimums = Vec::from_array(&e, [amt_a_min, amt_b_min]);

        let amounts = pool.withdraw(
            &to,
            &lp_amt,
            &minimums,
        );
        let amt_a = amounts.get(0).unwrap() as i128;
        let amt_b = amounts.get(1).unwrap() as i128;
        bump(&e);
        Ok((amt_a, amt_b))
    }
    /* ---------- quotes ---------- */
    fn quote_in(e: Env, pool_address: Address, amount_in: i128, token_in: Address, token_out: Address) -> Result<i128, AdapterError> {
        if !is_init(&e) {
            return Err(AdapterError::NotInitialized);
        }
        if amount_in <= 0 {
            return Err(AdapterError::InvalidAmount);
        }
        let pool_client = protocol::AquaPoolClient::new(&e, &pool_address);
        let reserves = pool_client.get_reserves();
        let pool_tokens = pool_client.get_tokens();
        let (reserve_in, reserve_out) = if pool_tokens.get(0).unwrap() == token_in {
            (reserves.get(0).unwrap(), reserves.get(1).unwrap())
        } else {
            (reserves.get(1).unwrap(), reserves.get(0).unwrap())
        };
        if reserve_in == 0 || reserve_out == 0 {
            return Ok(0); // No liquidity
        }
        // Standard constant product formula
        let amount_out = (amount_in as u128 * reserve_out) / (reserve_in + amount_in as u128);
        Ok(amount_out as i128)
    }

    fn quote_out(e: Env, pool_address: Address, amount_out: i128, token_in: Address, token_out: Address) -> Result<i128, AdapterError> {
        if !is_init(&e) {
            return Err(AdapterError::NotInitialized);
        }
        if amount_out <= 0 {
            return Err(AdapterError::InvalidAmount);
        }
        let pool_client = protocol::AquaPoolClient::new(&e, &pool_address);
        let reserves = pool_client.get_reserves();
        let pool_tokens = pool_client.get_tokens();
        let (reserve_in, reserve_out) = if pool_tokens.get(0).unwrap() == token_in {
            (reserves.get(0).unwrap(), reserves.get(1).unwrap())
        } else {
            (reserves.get(1).unwrap(), reserves.get(0).unwrap())
        };

        if reserve_in == 0 || reserve_out == 0 || reserve_out <= amount_out as u128 {
            return Err(AdapterError::InsufficientLiquidity);
        }

        // Standard constant product formula for required input
        let amount_in = (reserve_in * amount_out as u128) / (reserve_out - amount_out as u128) + 1;

        Ok(amount_in as i128)
    }
}

/// Direct pool operations called by the Router when tokens are pre-transferred.
/// Shallow-auth pattern: adapter is the direct caller, no require_auth on `to`.
#[contractimpl]
impl AquaAdapter {
    pub fn swap_in_pool(
        e: Env,
        amt_in: i128,
        min_out: i128,
        token_in: Address,
        token_out: Address,
        pool: Address,
        to: Address,
    ) -> i128 {
        // Tokens are already in this adapter (transferred by Router).
        // Transfer from adapter to the pool - adapter is direct caller -> auth works.
        token::Client::new(&e, &token_in)
            .transfer(&e.current_contract_address(), &pool, &amt_in);

        // Determine token indices within the pool
        let pool_client = protocol::AquaPoolClient::new(&e, &pool);
        let tokens = pool_client.get_tokens();
        let in_idx: u32 = if tokens.get(0).unwrap() == token_in { 0 } else { 1 };
        let out_idx: u32 = if in_idx == 0 { 1 } else { 0 };

        // Execute swap via Aqua pool - sends output tokens directly to `to`
        let amt_out = pool_client.swap(
            &to,
            &in_idx,
            &out_idx,
            &(amt_in as u128),
            &(min_out as u128),
        );

        let amt_out_i128 = amt_out as i128;

        event::swap(
            &e,
            event::SwapEvent {
                amt_in,
                amt_out: amt_out_i128,
                path: Vec::from_array(&e, [token_in, token_out]),
                to,
            },
        );
        bump(&e);

        amt_out_i128
    }

    pub fn add_liq_in_pool(
        e: Env,
        token_a: Address,
        token_b: Address,
        amount_a: i128,
        amount_b: i128,
        pool: Address,
        to: Address,
    ) -> i128 {
        // Tokens are in this adapter (pre-transferred by Router).
        // Aqua pool.deposit() calls require_auth(user) and pulls tokens from user,
        // so we deposit as the ADAPTER and forward LP tokens to the recipient.

        let pool_client = protocol::AquaPoolClient::new(&e, &pool);
        let reserves = pool_client.get_reserves();
        let reserve_a = reserves.get(0).unwrap();
        let reserve_b = reserves.get(1).unwrap();
        let total_shares = pool_client.get_total_shares();

        let (dep_a, dep_b) = get_deposit_amounts(
            &e,
            amount_a as u128,
            0u128,
            amount_b as u128,
            0u128,
            reserve_a,
            reserve_b,
        );

        let min_shares = get_shares(&e, dep_a, dep_b, reserve_a, reserve_b, total_shares);
        let desired_amounts = Vec::from_array(&e, [dep_a, dep_b]);

        // The adapter is the direct invoker of pool.deposit, so the pool's
        // user.require_auth() is auto-satisfied. We only need to authorize
        // the token transfers that the pool makes on our behalf (indirect calls).
        e.authorize_as_current_contract(soroban_sdk::vec![
            &e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: token_a.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: (
                        e.current_contract_address(),
                        pool.clone(),
                        dep_a as i128,
                    )
                        .into_val(&e),
                },
                sub_invocations: soroban_sdk::vec![&e],
            }),
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: token_b.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: (
                        e.current_contract_address(),
                        pool.clone(),
                        dep_b as i128,
                    )
                        .into_val(&e),
                },
                sub_invocations: soroban_sdk::vec![&e],
            }),
        ]);

        // Deposit as the adapter — pool pulls tokens and mints shares to adapter.
        // LP shares intentionally stay in the adapter: transferring reward-bearing
        // Aqua LP tokens triggers 4 checkpoint rounds (checkpoint_reward ×2 +
        // checkpoint_working_balance ×2) that exceed Soroban's per-tx memory budget.
        let (_amounts, shares_minted) =
            pool_client.deposit(&e.current_contract_address(), &desired_amounts, &min_shares);

        event::add_lp(
            &e,
            event::AddLpEvent {
                token_a,
                token_b,
                lp: pool,
                to,
            },
        );
        bump(&e);

        shares_minted as i128
    }
}
