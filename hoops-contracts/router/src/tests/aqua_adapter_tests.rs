// Aqua Adapter tests: swap and liquidity
#![cfg(test)]
use crate::tests::test_setup::{aqua_pool_constant::AquaPoolClient, HoopsTestEnvironment};
use soroban_sdk::token;
#[allow(unused_imports)]
use soroban_sdk::{vec, Env};
use soroban_sdk::testutils::Logs;
extern crate std;

pub fn run_swap_exact_in(test_env: &HoopsTestEnvironment) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let aqua_adapter_client = &test_env.adapters.aqua;
    let amount_in: i128 = 1_000_000;
    let amount_out_min: i128 = 0;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![
        env,
        token_a_client.address.clone(),
        token_b_client.address.clone(),
    ];
    token_a_client.approve(
        &user,
        &test_env.aqua.router_id.clone().unwrap(),
        &amount_in,
        &(env.ledger().timestamp() as u32 + 200),
    );
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!(
        "[AQUA][swap_exact_in] Initial user balances: TKA = {}, TKB = {}",
        initial_user_balance_a,
        initial_user_balance_b
    );
    let amount_out =
        aqua_adapter_client.swap_exact_in(&amount_in, &amount_out_min, &path, user, &deadline);
    std::println!(
        "[AQUA][swap_exact_in] Swap result: amount_in = {}, amount_out = {}",
        amount_in,
        amount_out
    );
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!(
        "[AQUA][swap_exact_in] Final user balances: TKA = {}, TKB = {}",
        final_user_balance_a,
        final_user_balance_b
    );
    assert!(
        amount_out > amount_out_min,
        "Amount out should be greater than min_out"
    );
    assert_eq!(
        final_user_balance_a,
        initial_user_balance_a - amount_in,
        "User TKA balance should decrease by amount_in"
    );
    assert_eq!(
        final_user_balance_b,
        initial_user_balance_b + amount_out,
        "User TKB balance should increase by amount_out"
    );
}

pub fn run_swap_exact_out(test_env: &HoopsTestEnvironment) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let aqua_adapter_client = &test_env.adapters.aqua;
    let desired_out: i128 = 500_000;
    let max_in: i128 = 2_000_000;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![
        env,
        token_a_client.address.clone(),
        token_b_client.address.clone(),
    ];
    token_a_client.approve(
        &user,
        &test_env.aqua.router_id.clone().unwrap(),
        &max_in,
        &(env.ledger().timestamp() as u32 + 200),
    );
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!(
        "[AQUA][swap_exact_out] Initial user balances: TKA = {}, TKB = {}",
        initial_user_balance_a,
        initial_user_balance_b
    );
    let amount_in_used =
        aqua_adapter_client.swap_exact_out(&desired_out, &max_in, &path, user, &deadline);
    std::println!(
        "[AQUA][swap_exact_out] Swap result: max_in = {}, actual_in = {}, desired_out = {}",
        max_in,
        amount_in_used,
        desired_out
    );
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!(
        "[AQUA][swap_exact_out] Final user balances: TKA = {}, TKB = {}",
        final_user_balance_a,
        final_user_balance_b
    );
    assert!(amount_in_used <= max_in, "Should not use more than max_in");
    assert_eq!(
        final_user_balance_b,
        initial_user_balance_b + desired_out,
        "User TKB balance should increase by desired_out"
    );
    assert_eq!(
        final_user_balance_a,
        initial_user_balance_a - amount_in_used,
        "User TKA balance should decrease by amount_in_used"
    );
}

pub fn run_add_liquidity(test_env: &HoopsTestEnvironment) -> i128 {
    let lp = 0;
    let env = &test_env.env;
    let logs = env.logs().all();
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let aqua_adapter_client = &test_env.adapters.aqua;

    let deadline = env.ledger().timestamp() + 100;
    let pool = test_env.aqua.pool_ids.get(0).unwrap();
    // Query pool reserves
    let pool_client = AquaPoolClient::new(&env, &pool);
    let reserves = pool_client.get_reserves();
    std::println!(
        "[AQUA][add_liquidity] Pool reserves: {:?}",
        reserves
    );
    let total_shares = pool_client.get_total_shares();
    std::println!(
        "[AQUA][add_liquidity] Total shares in pool: {}",
        total_shares
    );
    let reserve_a = reserves.get(0).unwrap() as i128;
    let reserve_b = reserves.get(1).unwrap() as i128;

    // Choose a reasonable deposit scale (e.g., 10 units)
    let scale = 100_000_000i128; // 10 * 10^7

    // Match the pool's ratio
    let ratio = reserve_b as f64 / reserve_a as f64;
    let amount_a: i128 = scale;
    let amount_b: i128 = (scale as f64 * ratio) as i128;

    // Set minimums to 99% of deposit
    let amt_a_min: i128 = (amount_a as f64 * 0.9) as i128;
    let amt_b_min: i128 = (amount_b as f64 * 0.9) as i128;
    token_a_client.approve(
        user,
        &pool,
        &amount_a,
        &(env.ledger().timestamp() as u32 + 200),
    );
    token_b_client.approve(
        user,
        &pool,
        &amount_b,
        &(env.ledger().timestamp() as u32 + 200),
    );
    let before_balance_a = token_a_client.balance(user);
    let before_balance_b = token_b_client.balance(user);
    std::println!(
        "[AQUA][add_liquidity] User balances before: TKA = {}, TKB = {}",
        before_balance_a,
        before_balance_b
    );
    let (amt_a, amt_b, lp) = aqua_adapter_client.add_liquidity(
        &token_a_client.address,
        &token_b_client.address,
        &amount_a,
        &amount_b,
        &amt_a_min,
        &amt_b_min,
        user,
        &deadline,
    );
    std::println!(
        "[AQUA][add_liquidity] Result: amt_a = {}, amt_b = {}, lp = {}",
        amt_a,
        amt_b,
        lp
    );
    let after_balance_a = token_a_client.balance(user);
    let after_balance_b = token_b_client.balance(user);
    std::println!(
        "[AQUA][add_liquidity] User balances after: TKA = {}, TKB = {}",
        after_balance_a,
        after_balance_b
    );
    assert!(
        amt_a > 0 && amt_b > 0 && lp > 0,
        "Should add liquidity and receive LP tokens"
    );
    assert!(
        after_balance_a < before_balance_a,
        "User TKA balance should decrease"
    );
    assert!(
        after_balance_b < before_balance_b,
        "User TKB balance should decrease"
    );
    std::println!("{}", logs.join("\n"));
    lp
}

pub fn run_remove_liquidity(test_env: &HoopsTestEnvironment, lp: i128) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let aqua_adapter_client = &test_env.adapters.aqua;
    let pool = test_env.aqua.pool_ids.get(0).unwrap();
    let pool_client = AquaPoolClient::new(&env, &pool);
    let share_id = pool_client.share_id();
    let lp_amt: i128 = lp;
    let amt_a_min: i128 = 1;
    let amt_b_min: i128 = 1;
    let deadline = env.ledger().timestamp() + 100;
    std::println!(
        "[AQUA][remove_liquidity] Removing liquidity: lp_amt = {}",
        lp_amt
    );
    let (amt_a_out, amt_b_out) = aqua_adapter_client
        .remove_liquidity(&share_id, &lp_amt, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!(
        "[AQUA][remove_liquidity] Result: amt_a_out = {}, amt_b_out = {}",
        amt_a_out,
        amt_b_out
    );
    assert!(
        amt_a_out > 0 || amt_b_out > 0,
        "Should withdraw some tokens"
    );
}

pub fn test_aqua_adapter(test_env: &HoopsTestEnvironment, failures: i32) -> i32 {
    use std::panic::AssertUnwindSafe;
    let mut failures = failures;
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_swap_exact_in(&test_env))) {
        std::println!("[FAIL][AQUA  ][swap_exact_in]: {:?}", e);
        failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_swap_exact_out(&test_env))) {
        std::println!("[FAIL][AQUA][swap_exact_out]: {:?}", e);
        failures += 1;
    }
    // Run add_liquidity and capture the LP token amount
    let lp = match std::panic::catch_unwind(AssertUnwindSafe(|| run_add_liquidity(&test_env))) {
        Ok(lp) => lp,
        Err(e) => {
            std::println!("[FAIL][AQUA][add_liquidity]: {:?}", e);
            failures += 1;
            0 // Default value if the test fails
        }
    };
    // Only run remove_liquidity if add_liquidity succeeded (lp > 0)
    if lp > 0 {
        if let Err(e) =
            std::panic::catch_unwind(AssertUnwindSafe(|| run_remove_liquidity(&test_env, lp)))
        {
            std::println!("[FAIL][AQUA][remove_liquidity]: {:?}", e);
            failures += 1;
        }
    } else {
        std::println!("[INFO][AQUA] add liquidity failed, skipping remove");
    }
    std::println!("[AQUA] Test results: {} failures", failures);
    if failures > 0 {
        panic!(
            "{} Aqua adapter subtests failed. See log for details.",
            failures
        );
    }
    failures
}

#[test]
pub fn test_aqua_adapter_all() {
    let test_env = HoopsTestEnvironment::setup();
    let mut failures = 0;
    failures += test_aqua_adapter(&test_env, failures);
    std::println!(
        "[TEST] Aqua adapter tests completed with {} failures",
        failures
    );
    assert!(
        failures == 0,
        "{:?} Aqua adapter tests failed. See log for details.",
        failures
    );
}
