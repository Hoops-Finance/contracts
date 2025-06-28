// Phoenix Adapter tests: swap and liquidity
#![cfg(test)]
use soroban_sdk::token;
#[allow(unused_imports)]
use soroban_sdk::{Env, vec};

use crate::tests::test_setup::{PhoenixPoolClient, HoopsTestEnvironment, phoenix_pool};
extern crate std;


fn get_pool_info(env: &soroban_sdk::Env, pool_addr: &soroban_sdk::Address) -> phoenix_pool::PoolResponse {
    let pool = PhoenixPoolClient::new(env, pool_addr);
    pool.query_pool_info()
}

fn percent_of(val: i128, percent: i128) -> i128 {
    (val * percent) / 100
}

pub fn run_swap_exact_in(test_env: &HoopsTestEnvironment) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let phoenix_adapter_client = &test_env.adapters.phoenix;
    let pool_addr = &test_env.phoenix.pool_ids.get(0).unwrap();
    let pool_info = get_pool_info(env, &pool_addr);
    let reserve_a = pool_info.asset_a.amount;
    let reserve_b = pool_info.asset_b.amount;
    let amount_in: i128 = percent_of(reserve_a, 5); // 5% of reserve
    let slippage_bps: i128 = 50; // 0.5%
    // XYK AMM math: out = (amount_in * reserve_b * 997) / (reserve_a * 1000 + amount_in * 997)
    let numerator = amount_in * reserve_b * 997;
    let denominator = reserve_a * 1000 + amount_in * 997;
    let expected_out = numerator / denominator;
    let min_out = expected_out * (10_000 - slippage_bps) / 10_000;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    token_a_client.approve(user, &pool_addr, &amount_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[PHOENIX][swap_exact_in] Initial user balances: TKA = {:.7}, TKB = {:.7}", initial_user_balance_a as f64 * 1e-7, initial_user_balance_b as f64 * 1e-7);
    std::println!("[PHOENIX][swap_exact_in] Pool reserves: TKA = {:.7}, TKB = {:.7}", reserve_a as f64 * 1e-7, reserve_b as f64 * 1e-7);
    std::println!("[PHOENIX][swap_exact_in] amount_in = {:.7}, min_out = {:.7}, expected_out = {:.7}, slippage_bps = {}", amount_in as f64 * 1e-7, min_out as f64 * 1e-7, expected_out as f64 * 1e-7, slippage_bps);
    let amount_out = phoenix_adapter_client.swap_exact_in(&amount_in, &min_out, &path, user, &deadline);
    std::println!("[PHOENIX][swap_exact_in] Swap result: amount_in = {:.7}, amount_out = {:.7}", amount_in as f64 * 1e-7, amount_out as f64 * 1e-7);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[PHOENIX][swap_exact_in] Final user balances: TKA = {:.7}, TKB = {:.7}", final_user_balance_a as f64 * 1e-7, final_user_balance_b as f64 * 1e-7);
    assert!(amount_out >= min_out, "Amount out should be >= min_out");
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in, "User TKA balance should decrease by amount_in");
    assert_eq!(final_user_balance_b, initial_user_balance_b + amount_out, "User TKB balance should increase by amount_out");
}

pub fn run_swap_exact_out(test_env: &HoopsTestEnvironment) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let phoenix_adapter_client = &test_env.adapters.phoenix;
    let pool_addr = &test_env.phoenix.pool_ids.get(0).unwrap();
    let pool_info = get_pool_info(env, pool_addr);
    let reserve_a = pool_info.asset_a.amount;
    let reserve_b = pool_info.asset_b.amount;
    let desired_out: i128 = percent_of(reserve_b, 3); // 3% of reserve
    let slippage_bps: i128 = 50; // 0.5%
    // XYK AMM math: in = (reserve_a * desired_out * 1000) / ((reserve_b - desired_out) * 997) + 1
    let numerator = reserve_a * desired_out * 1000;
    let denominator = (reserve_b - desired_out) * 997;
    let expected_in = numerator / denominator + 1;
    let max_in = expected_in * (10_000 + slippage_bps) / 10_000;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    token_a_client.approve(user, pool_addr, &max_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[PHOENIX][swap_exact_out] Initial user balances: TKA = {:.7}, TKB = {:.7}", initial_user_balance_a as f64 * 1e-7, initial_user_balance_b as f64 * 1e-7);
    std::println!("[PHOENIX][swap_exact_out] Pool reserves: TKA = {:.7}, TKB = {:.7}", reserve_a as f64 * 1e-7, reserve_b as f64 * 1e-7);
    std::println!("[PHOENIX][swap_exact_out] desired_out = {:.7}, max_in = {:.7}, expected_in = {:.7}, slippage_bps = {}", desired_out as f64 * 1e-7, max_in as f64 * 1e-7, expected_in as f64 * 1e-7, slippage_bps);
    let amount_in_used = phoenix_adapter_client.swap_exact_out(&desired_out, &max_in, &path, user, &deadline);
    std::println!("[PHOENIX][swap_exact_out] Swap result: max_in = {:.7}, actual_in = {:.7}, desired_out = {:.7}", max_in as f64 * 1e-7, amount_in_used as f64 * 1e-7, desired_out as f64 * 1e-7);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[PHOENIX][swap_exact_out] Final user balances: TKA = {:.7}, TKB = {:.7}", final_user_balance_a as f64 * 1e-7, final_user_balance_b as f64 * 1e-7);
    assert!(amount_in_used <= max_in, "Should not use more than max_in");
    assert_eq!(final_user_balance_b, initial_user_balance_b + desired_out, "User TKB balance should increase by desired_out");
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in_used, "User TKA balance should decrease by amount_in_used");
}

// Store LP minted in add_liquidity for use in remove_liquidity
pub fn run_add_liquidity(test_env: &HoopsTestEnvironment) -> i128 {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let phoenix_adapter_client = &test_env.adapters.phoenix;
    let pool_addr = &test_env.phoenix.pool_ids.get(0).unwrap();
    let pool_info = get_pool_info(env, pool_addr);
    let reserve_a = pool_info.asset_a.amount;
    let reserve_b = pool_info.asset_b.amount;
    let lp_supply = pool_info.asset_lp_share.amount;
    let amount_a: i128 = percent_of(reserve_a, 5); // 5% of reserve
    let amount_b: i128 = percent_of(reserve_b, 5); // 5% of reserve
    let slippage_bps: i128 = 50; // 0.5%
    let amt_a_min = amount_a * (10_000 - slippage_bps) / 10_000;
    let amt_b_min = amount_b * (10_000 - slippage_bps) / 10_000;
    let deadline = env.ledger().timestamp() + 100;
    token_a_client.approve(user, pool_addr, &amount_a, &(env.ledger().timestamp() as u32 + 200));
    token_b_client.approve(user, pool_addr, &amount_b, &(env.ledger().timestamp() as u32 + 200));
    let before_balance_a = token_a_client.balance(user);
    let before_balance_b = token_b_client.balance(user);
    let before_lp = token::TokenClient::new(env, &pool_info.asset_lp_share.address).balance(user);
    std::println!("[PHOENIX][add_liquidity] User balances before: TKA = {:.7}, TKB = {:.7}, LP = {:.7}", before_balance_a as f64 * 1e-7, before_balance_b as f64 * 1e-7, before_lp as f64 * 1e-7);
    std::println!("[PHOENIX][add_liquidity] Pool reserves: TKA = {:.7}, TKB = {:.7}, LP supply = {:.7}", reserve_a as f64 * 1e-7, reserve_b as f64 * 1e-7, lp_supply as f64 * 1e-7);
    std::println!("[PHOENIX][add_liquidity] amount_a = {:.7}, amount_b = {:.7}, amt_a_min = {:.7}, amt_b_min = {:.7}, slippage_bps = {}", amount_a as f64 * 1e-7, amount_b as f64 * 1e-7, amt_a_min as f64 * 1e-7, amt_b_min as f64 * 1e-7, slippage_bps);
    let (amt_a, amt_b, lp) = phoenix_adapter_client.add_liquidity(&token_a_client.address, &token_b_client.address, &amount_a, &amount_b, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[PHOENIX][add_liquidity] Result: amt_a = {:.7}, amt_b = {:.7}, lp = {:.7}", amt_a as f64 * 1e-7, amt_b as f64 * 1e-7, lp as f64 * 1e-7);
    let after_balance_a = token_a_client.balance(user);
    let after_balance_b = token_b_client.balance(user);
    let after_lp = token::TokenClient::new(env, &pool_info.asset_lp_share.address).balance(user);
    std::println!("[PHOENIX][add_liquidity] User balances after: TKA = {:.7}, TKB = {:.7}, LP = {:.7}", after_balance_a as f64 * 1e-7, after_balance_b as f64 * 1e-7, after_lp as f64 * 1e-7);
    assert!(amt_a > 0 && amt_b > 0, "Should add liquidity");
    assert!(after_balance_a < before_balance_a, "User TKA balance should decrease");
    assert!(after_balance_b < before_balance_b, "User TKB balance should decrease");
    assert!(after_lp > before_lp, "User LP balance should increase");
    lp
}

pub fn run_remove_liquidity(test_env: &HoopsTestEnvironment, lp_amt: i128) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let phoenix_adapter_client = &test_env.adapters.phoenix;
    let pool_addr = &test_env.phoenix.pool_ids.get(0).unwrap();
    let pool_info = get_pool_info(env, pool_addr);
    let lp_token = pool_info.asset_lp_share.address.clone();
    let user_lp_balance = token::TokenClient::new(env, &lp_token).balance(user);
    let slippage_bps: i128 = 50; // 0.5%
    // Calculate expected out based on pool math (proportional to LP burned)
    let reserve_a = pool_info.asset_a.amount;
    let reserve_b = pool_info.asset_b.amount;
    let lp_supply = pool_info.asset_lp_share.amount;
    let expected_a = reserve_a * lp_amt / lp_supply;
    let expected_b = reserve_b * lp_amt / lp_supply;
    let amt_a_min = expected_a * (10_000 - slippage_bps) / 10_000;
    let amt_b_min = expected_b * (10_000 - slippage_bps) / 10_000;
    let deadline = env.ledger().timestamp() + 100;
    std::println!("[PHOENIX][remove_liquidity] Removing liquidity: lp_amt = {:.7} (user LP balance = {:.7})", lp_amt as f64 * 1e-7, user_lp_balance as f64 * 1e-7);
    std::println!("[PHOENIX][remove_liquidity] Expected out: amt_a = {:.7}, amt_b = {:.7}, min: amt_a_min = {:.7}, amt_b_min = {:.7}", expected_a as f64 * 1e-7, expected_b as f64 * 1e-7, amt_a_min as f64 * 1e-7, amt_b_min as f64 * 1e-7);
    let (amt_a_out, amt_b_out) = phoenix_adapter_client.remove_liquidity(&pool_addr, &lp_amt, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[PHOENIX][remove_liquidity] Result: amt_a_out = {:.7}, amt_b_out = {:.7}", amt_a_out as f64 * 1e-7, amt_b_out as f64 * 1e-7);
    let after_lp = token::TokenClient::new(env, &lp_token).balance(user);
    std::println!("[PHOENIX][remove_liquidity] User LP balance after: {:.7}", after_lp as f64 * 1e-7);
    assert!(amt_a_out > 0 || amt_b_out > 0, "Should withdraw some tokens");
    assert!(after_lp < user_lp_balance, "User LP balance should decrease");
}

#[test]
pub fn test_phoenix_adapter_all() {
    use std::panic::AssertUnwindSafe;
    let test_env = HoopsTestEnvironment::setup();
    let mut failures = 0;
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_swap_exact_in(&test_env))) {
        std::println!("[FAIL][PHOENIX][swap_exact_in]: {:?}", e); failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_swap_exact_out(&test_env))) {
        std::println!("[FAIL][PHOENIX][swap_exact_out]: {:?}", e); failures += 1;
    }
    // Run add_liquidity and capture the LP token amount
    let lp = match std::panic::catch_unwind(AssertUnwindSafe(|| run_add_liquidity(&test_env))) {
        Ok(lp) => lp,
        Err(e) => {
            std::println!("[FAIL][PHOENIX][add_liquidity]: {:?}", e); failures += 1;
            0 // Default value if the test fails
        }
    };
    // Only run remove_liquidity if add_liquidity succeeded (lp > 0)
    if lp > 0 {
        if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_remove_liquidity(&test_env, lp))) {
            std::println!("[FAIL][PHOENIX][remove_liquidity]: {:?}", e); failures += 1;
        }
    } else {
        std::println!("[INFO][PHOENIX] add liquidity failed, skipping remove");
    }
    std::println!("[PHOENIX] Test results: {} failures", failures);
    if failures > 0 {
        panic!("{} Phoenix adapter subtests failed. See log for details.", failures);
    }
}
