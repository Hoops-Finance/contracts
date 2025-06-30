#![cfg(test)]
use soroban_sdk::token;
#[allow(unused_imports)]
use soroban_sdk::{Env, vec};
use crate::tests::test_setup::{ HoopsTestEnvironment};
extern crate std;

pub fn run_swap_exact_in(test_env: &HoopsTestEnvironment) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let soroswap_adapter_client = &test_env.adapters.soroswap;
    let pool = test_env.soroswap.pool_ids.get(0).unwrap();
    let pool_client = crate::tests::test_setup::soroswap_pair::SoroswapPairClient::new(env, &pool);
    let reserves = pool_client.get_reserves();
    let amount_in: i128 = (reserves.0 / 20).max(1_000_000); // 5% of TKA reserve, at least 1 token
    // Calculate expected out using router's get_amounts_out
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    let router = test_env.soroswap.router_id.clone().unwrap();
    let router_client = crate::tests::test_setup::soroswap_router::SoroswapRouterClient::new(env, &router);
    let amounts_out = router_client.router_get_amounts_out(&amount_in, &path);
    let expected_out = amounts_out.get(1).unwrap_or(0i128);
    // Inline Uniswap/Soroswap math for min_out (with rounding -1)
    let reserve_in = reserves.0;
    let reserve_out = reserves.1;
    let fee_bps: i128 = 30; // 0.3% fee, adjust if different
    let amount_in_with_fee = amount_in * (10_000 - fee_bps);
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = reserve_in * 10_000 + amount_in_with_fee;
    let mut min_out = if denominator == 0 { 0 } else { numerator / denominator };
    if min_out > 0 { min_out -= 1; }
    let amount_out_min = min_out;
    let deadline = env.ledger().timestamp() + 100;
    token_a_client.approve(&user, &router, &amount_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[SOROSWAP][swap_exact_in] Initial user balances: TKA = {:.7}, TKB = {:.7}", (initial_user_balance_a as f64) * 1e-7, (initial_user_balance_b as f64) * 1e-7);
    std::println!("[SOROSWAP][swap_exact_in] Pool reserves: TKA = {:.7}, TKB = {:.7}", (reserves.0 as f64) * 1e-7, (reserves.1 as f64) * 1e-7);
    std::println!("[SOROSWAP][swap_exact_in] Using amount_in = {:.7}, min_out = {:.7} (AMM math, rounded)", (amount_in as f64) * 1e-7, (amount_out_min as f64) * 1e-7);
    let amount_out = soroswap_adapter_client.swap_exact_in(&amount_in, &amount_out_min, &path, user, &deadline);
    std::println!("[SOROSWAP][swap_exact_in] Swap result: amount_in = {:.7}, amount_out = {:.7}", (amount_in as f64) * 1e-7, (amount_out as f64) * 1e-7);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[SOROSWAP][swap_exact_in] Final user balances: TKA = {:.7}, TKB = {:.7}", (final_user_balance_a as f64) * 1e-7, (final_user_balance_b as f64) * 1e-7);
    assert!(amount_out >= amount_out_min, "Amount out should be at least min_out");
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in, "User TKA balance should decrease by amount_in");
    assert_eq!(final_user_balance_b, initial_user_balance_b + amount_out, "User TKB balance should increase by amount_out");
}

pub fn run_swap_exact_out(test_env: &HoopsTestEnvironment) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let soroswap_adapter_client = &test_env.adapters.soroswap;
    let pool = test_env.soroswap.pool_ids.get(0).unwrap();
    let pool_client = crate::tests::test_setup::soroswap_pair::SoroswapPairClient::new(env, &pool);
    let reserves = pool_client.get_reserves();
    let desired_out: i128 = (reserves.1 / 20).max(1_000_000); // 5% of TKB reserve, at least 1 token
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    // Calculate required input for desired_out using AMM math (no cross-contract call)
    let reserve_in = reserves.0;
    let reserve_out = reserves.1;
    let fee_bps: i128 = 30; // 0.3% fee, adjust if different
    let numerator = reserve_in * desired_out * 10_000;
    let denominator = (reserve_out - desired_out) * (10_000 - fee_bps);
    let max_in = if denominator == 0 { 0 } else { ((numerator + denominator ) / denominator) + 1 }; // ceiling division
    // For verification/logging only, get the router's computed value
    let router = test_env.soroswap.router_id.clone().unwrap();
    let router_client = crate::tests::test_setup::soroswap_router::SoroswapRouterClient::new(env, &router);
    let amounts_in = router_client.router_get_amounts_in(&desired_out, &path);
    let required_in = amounts_in.get(0).unwrap();
    std::println!("[SOROSWAP][swap_exact_out] Pool reserves: TKA = {:.7}, TKB = {:.7}", (reserves.0 as f64) * 1e-7, (reserves.1 as f64) * 1e-7);
    std::println!("[SOROSWAP][swap_exact_out] Using desired_out = {:.7} (5% of TKB reserve)", (desired_out as f64) * 1e-7);
    std::println!("[SOROSWAP][swap_exact_out] Required input for desired_out {:.7} is {:.7} (router), {:.7} (AMM math)", (desired_out as f64) * 1e-7, (required_in as f64) * 1e-7, (max_in as f64) * 1e-7);
    std::assert_eq!(max_in, required_in, "AMM math and router_get_amounts_in disagree!");
    std::println!("[SOROSWAP][swap_exact_out] Params: desired_out = {:.7}, max_in = {:.7}, deadline = {}", (desired_out as f64) * 1e-7, (max_in as f64) * 1e-7, deadline);
    std::println!("[SOROSWAP][swap_exact_out] Path: {:?}", path);
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[SOROSWAP][swap_exact_out] Initial user balances: TKA = {:.7}, TKB = {:.7}", (initial_user_balance_a as f64) * 1e-7, (initial_user_balance_b as f64) * 1e-7);
    let amount_in_used = soroswap_adapter_client.swap_exact_out(&desired_out, &max_in, &path, user, &deadline);
    std::println!("[SOROSWAP][swap_exact_out] SwapExactOut result: max_in = {:.7}, actual_in = {:.7}, desired_out = {:.7}", (max_in as f64) * 1e-7, (amount_in_used as f64) * 1e-7, (desired_out as f64) * 1e-7);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[SOROSWAP][swap_exact_out] Final user balances: TKA = {:.7}, TKB = {:.7}", (final_user_balance_a as f64) * 1e-7, (final_user_balance_b as f64) * 1e-7);
    assert!(amount_in_used <= max_in, "Should not use more than max_in");
    assert_eq!(final_user_balance_b, initial_user_balance_b + desired_out, "User TKB balance should increase by desired_out");
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in_used, "User TKA balance should decrease by amount_in_used");
}

pub fn run_add_liquidity(test_env: &HoopsTestEnvironment) -> i128 {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let soroswap_adapter_client = &test_env.adapters.soroswap;
    let pool = test_env.soroswap.pool_ids.get(0).unwrap();
    let pool_client = crate::tests::test_setup::soroswap_pair::SoroswapPairClient::new(env, &pool);
    let lp_token_client = token::Client::new(env, &pool);
    let before_reserves = pool_client.get_reserves();
    let before_lp_balance = lp_token_client.balance(user);
    // Use 5% of each reserve for add
    let amount_a: i128 = (before_reserves.0 / 20).max(1_000_000);
    let amount_b: i128 = (before_reserves.1 / 20).max(1_000_000);
    let amt_a_min: i128 = 1; // Patch: supply min amounts
    let amt_b_min: i128 = 1;
    let deadline = env.ledger().timestamp() + 100;
    std::println!("[SOROSWAP][add_liquidity] Pool reserves before: TKA = {:.7}, TKB = {:.7}", (before_reserves.0 as f64) * 1e-7, (before_reserves.1 as f64) * 1e-7);
    std::println!("[SOROSWAP][add_liquidity] User LP balance before: {:.7}", (before_lp_balance as f64) * 1e-7);
    std::println!("[SOROSWAP][add_liquidity] Adding liquidity: amount_a = {:.7}, amount_b = {:.7}", (amount_a as f64) * 1e-7, (amount_b as f64) * 1e-7);
    token_a_client.approve(user, &test_env.soroswap.router_id.clone().unwrap(), &amount_a, &(env.ledger().timestamp() as u32 + 200));
    token_b_client.approve(user, &test_env.soroswap.router_id.clone().unwrap(), &amount_b, &(env.ledger().timestamp() as u32 + 200));
    // Calculate min_lp_out (no fee, proportional to supply)
    let total_lp = pool_client.total_supply();
    let min_lp_out = if total_lp == 0 {
        // Initial liquidity, just use 1 as min
        1
    } else {
        let lp_a = amount_a * total_lp / before_reserves.0;
        let lp_b = amount_b * total_lp / before_reserves.1;
        let mut min_lp = lp_a.min(lp_b);
        if min_lp > 0 { min_lp -= 1; }
        min_lp
    };
    let (amt_a, amt_b, lp) = soroswap_adapter_client.add_liquidity(&token_a_client.address, &token_b_client.address, &amount_a, &amount_b, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[SOROSWAP][add_liquidity] min_lp_out (calculated) = {:.7}", (min_lp_out as f64) * 1e-7);
    std::println!("[SOROSWAP][add_liquidity] Result: amt_a = {:.7}, amt_b = {:.7}, lp = {:.7}", (amt_a as f64) * 1e-7, (amt_b as f64) * 1e-7, (lp as f64) * 1e-7);
    let after_reserves = pool_client.get_reserves();
    let after_lp_balance = lp_token_client.balance(user);
    std::println!("[SOROSWAP][add_liquidity] Pool reserves after: TKA = {:.7}, TKB = {:.7}", (after_reserves.0 as f64) * 1e-7, (after_reserves.1 as f64) * 1e-7);
    std::println!("[SOROSWAP][add_liquidity] User LP balance after: {:.7}", (after_lp_balance as f64) * 1e-7);
    assert!(amt_a > 0 && amt_b > 0 && lp > 0, "Should add liquidity and receive LP tokens");
    assert!(after_reserves.0 > before_reserves.0 || after_reserves.1 > before_reserves.1, "Pool reserves should increase");
    assert!(after_lp_balance > before_lp_balance, "User LP balance should increase");
    lp
}

pub fn run_remove_liquidity(test_env: &HoopsTestEnvironment, lp_amt: i128) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let soroswap_adapter_client = &test_env.adapters.soroswap;
    let pool = test_env.soroswap.pool_ids.get(0).unwrap();
    let pool_client = crate::tests::test_setup::soroswap_pair::SoroswapPairClient::new(env, &pool);
    let lp_token_client = token::Client::new(env, &pool);
    let before_reserves = pool_client.get_reserves();
    let before_lp_balance = lp_token_client.balance(user);
    // Calculate expected out amounts based on share of LP (no fee)
    let total_lp = pool_client.total_supply();
    let mut expected_a = before_reserves.0.saturating_mul(lp_amt) / total_lp;
    let mut expected_b = before_reserves.1.saturating_mul(lp_amt) / total_lp;
    if expected_a > 0 { expected_a -= 1; }
    if expected_b > 0 { expected_b -= 1; }
    let amt_a_min: i128 = expected_a;
    let amt_b_min: i128 = expected_b;
    let deadline = env.ledger().timestamp() + 100;
    std::println!("[SOROSWAP][remove_liquidity] Pool reserves before: TKA = {:.7}, TKB = {:.7}", (before_reserves.0 as f64) * 1e-7, (before_reserves.1 as f64) * 1e-7);
    std::println!("[SOROSWAP][remove_liquidity] User LP balance before: {:.7}", (before_lp_balance as f64) * 1e-7);
    std::println!("[SOROSWAP][remove_liquidity] Removing liquidity: lp_amt = {:.7} (min out: TKA = {:.7}, TKB = {:.7})", (lp_amt as f64) * 1e-7, (amt_a_min as f64) * 1e-7, (amt_b_min as f64) * 1e-7);
    let (amt_a_out, amt_b_out) = soroswap_adapter_client.remove_liquidity(&pool, &lp_amt, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[SOROSWAP][remove_liquidity] Result: amt_a_out = {:.7}, amt_b_out = {:.7}", (amt_a_out as f64) * 1e-7, (amt_b_out as f64) * 1e-7);
    let after_reserves = pool_client.get_reserves();
    let after_lp_balance = lp_token_client.balance(user);
    std::println!("[SOROSWAP][remove_liquidity] Pool reserves after: TKA = {:.7}, TKB = {:.7}", (after_reserves.0 as f64) * 1e-7, (after_reserves.1 as f64) * 1e-7);
    std::println!("[SOROSWAP][remove_liquidity] User LP balance after: {:.7}", (after_lp_balance as f64) * 1e-7);
    assert!(amt_a_out >= amt_a_min, "Should withdraw at least min TKA");
    assert!(amt_b_out >= amt_b_min, "Should withdraw at least min TKB");
    assert!(after_reserves.0 < before_reserves.0 || after_reserves.1 < before_reserves.1, "Pool reserves should decrease");
    assert!(after_lp_balance < before_lp_balance, "User LP balance should decrease");
}

pub fn test_soroswap_adapter(test_env: &HoopsTestEnvironment, failures: i32) -> i32 {
    use std::panic::AssertUnwindSafe;
    let mut failures = failures;
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_swap_exact_in(&test_env))) {
        std::println!("[FAIL][SOROSWAP][swap_exact_in]: {:?}", e); failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_swap_exact_out(&test_env))) {
        std::println!("[FAIL][SOROSWAP][swap_exact_out]: {:?}", e); failures += 1;
    }
    // Run add_liquidity and capture the LP token amount
    let lp = match std::panic::catch_unwind(AssertUnwindSafe(|| run_add_liquidity(&test_env))) {
        Ok(lp) => lp,
        Err(e) => {
            std::println!("[FAIL][SOROSWAP][add_liquidity]: {:?}", e); failures += 1;
            0 // Default value if the test fails
        }
    };
    // Only run remove_liquidity if add_liquidity succeeded (lp > 0)
    if lp > 0 {
        if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_remove_liquidity(&test_env, lp))) {
            std::println!("[FAIL][SOROSWAP][remove_liquidity]: {:?}", e); failures += 1;
        }
    } else {
        std::println!("[INFO][SOROSWAP] add liquidity failed, skipping remove");
    }
    std::println!("[SOROSWAP] Test results: {} failures", failures);
    if failures > 0 {
        panic!("{} Soroswap adapter subtests failed. See log for details.", failures);
    }

    failures
}


#[test]
pub fn test_soroswap_adapter_all() {
    let test_env = HoopsTestEnvironment::setup();
    let mut failures = 0;
   failures += test_soroswap_adapter(&test_env, failures);
    assert_eq!(failures, 0, "Soroswap adapter tests failed with {} errors", failures);
}