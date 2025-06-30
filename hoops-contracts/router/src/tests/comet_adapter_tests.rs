// Comet Adapter tests: swap and liquidity
#![cfg(test)]
use core::panic::AssertUnwindSafe;

use soroban_sdk::token;
#[allow(unused_imports)]
use soroban_sdk::{Env, vec, Address, Vec};
use crate::tests::test_setup::comet_pool::CometPoolClient;
use crate::tests::test_setup::HoopsTestEnvironment;
use crate::tests::test_setup::comet_adapter::Client as CometAdapterClient;
extern crate std;

pub fn register_comet_pool(adapter: &CometAdapterClient, tokens: Vec<Address>, pool: Address) {
    adapter.set_pool_for_tokens(&tokens, &pool);
}

pub fn run_swap_exact_in(test_env: &HoopsTestEnvironment) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let comet_adapter_client = &test_env.adapters.comet;
    // Use a larger swap amount to ensure meaningful test
    let amount_in: i128 = 1_000_000_000; // 100 tokens
    let amount_out_min: i128 = 0;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    // Register pool for this token set
    let pool = test_env.comet.pool_ids.get(0).unwrap();
    register_comet_pool(comet_adapter_client, path.clone(), pool.clone());
    token_a_client.approve(&user, &pool, &amount_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[COMET][swap_exact_in] Initial user balances: TKA = {:.7}, TKB = {:.7}", (initial_user_balance_a as f64) * 1e-7, (initial_user_balance_b as f64) * 1e-7);
    let amount_out = comet_adapter_client.swap_exact_in(&amount_in, &amount_out_min, &path, user, &deadline);
    std::println!("[COMET][swap_exact_in] Swap result: amount_in = {}, amount_out = {}", amount_in, amount_out);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[COMET][swap_exact_in] Final user balances: TKA = {:.7}, TKB = {:.7}", (final_user_balance_a as f64) * 1e-7, (final_user_balance_b as f64) * 1e-7);
    assert!(amount_out > amount_out_min, "Amount out should be greater than min_out");
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in, "User TKA balance should decrease by amount_in");
    assert_eq!(final_user_balance_b, initial_user_balance_b + amount_out, "User TKB balance should increase by amount_out");
}

pub fn run_swap_exact_out(test_env: &HoopsTestEnvironment) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let comet_adapter_client = &test_env.adapters.comet;
    // Use a larger desired_out to ensure meaningful test
    let desired_out: i128 = 1_000_000_000; // 100 tokens (7 decimals)
    // Fetch pool spot price (with fee)
    let pool = CometPoolClient::new(env, &test_env.comet.pool_ids.get(0).unwrap());
    let spot = pool.get_spot_price(&token_a_client.address, &token_b_client.address); // i128, STROOP units
    let slippage_bps: i128 = 100; // 1% slippage
    let max_in = desired_out * spot * (10_000 + slippage_bps) / 10_000 / 10_000_000; // STROOP scaling
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    token_a_client.approve(&user, &test_env.comet.pool_ids.get(0).unwrap(), &max_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[COMET][swap_exact_out] Initial user balances: TKA = {:.7}, TKB = {:.7}", (initial_user_balance_a as f64) * 1e-7, (initial_user_balance_b as f64) * 1e-7);
    let amount_in_used = comet_adapter_client.swap_exact_out(&desired_out, &max_in, &path, user, &deadline);
    std::println!("[COMET][swap_exact_out] Swap result: max_in = {}, actual_in = {}, desired_out = {}", max_in, amount_in_used, desired_out);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[COMET][swap_exact_out] Final user balances: TKA = {:.7}, TKB = {:.7}", (final_user_balance_a as f64) * 1e-7, (final_user_balance_b as f64) * 1e-7);
    assert!(amount_in_used <= max_in, "Should not use more than max_in");
    assert_eq!(final_user_balance_b, initial_user_balance_b + desired_out, "User TKB balance should increase by desired_out");
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in_used, "User TKA balance should decrease by amount_in_used");
}

pub fn run_add_liquidity(test_env: &HoopsTestEnvironment) -> i128 {
    std::println!("\n[COMET][add_liquidity] Testing Add Liquidity via Comet Adapter");
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let comet_adapter_client = &test_env.adapters.comet;
    let pool = test_env.comet.pool_ids.get(0).unwrap();
    // Use 100 tokens (7 decimals)
    let amount_a: i128 = 1_000_000_000; // 100 tokens
    let pool_client = CometPoolClient::new(env, &pool);
    // Proportional amount_b
    let pool_balance_a = pool_client.get_balance(&token_a_client.address);
    let pool_balance_b = pool_client.get_balance(&token_b_client.address);
    std::println!("[COMET][add_liquidity] Pool balances: TKA = {:.7}, TKB = {:.7}", (pool_balance_a as f64) * 1e-7, (pool_balance_b as f64) * 1e-7);

    let spot_sans_fee = pool_client.get_spot_price_sans_fee(&token_a_client.address, &token_b_client.address);
    let implied_price = (pool_balance_a as f64) / (pool_balance_b as f64);
    std::println!("[COMET][add_liquidity] Spot price sans fee (A-in-B): {:.7} | Implied price (not considering weight) (A/B): {:.7}", (spot_sans_fee as f64) * 1e-7, implied_price);

    let amount_b = amount_a * pool_balance_b / pool_balance_a;
    let slippage_bps: i128 = 50; // 0.5% slippage
    let amt_a_min: i128 = amount_a * (10_000 - slippage_bps) / 10_000;
    let amt_b_min: i128 = amount_b * (10_000 - slippage_bps) / 10_000;
    let deadline = env.ledger().timestamp() + 100;
    token_a_client.approve(user, &pool, &amount_a, &(env.ledger().timestamp() as u32 + 200));
    token_b_client.approve(user, &pool, &amount_b, &(env.ledger().timestamp() as u32 + 200));
    let before_balance_a = token_a_client.balance(user);
    let before_balance_b = token_b_client.balance(user);
    std::println!("[COMET][add_liquidity] User balances before: TKA = {:.7}, TKB = {:.7}", (before_balance_a as f64) * 1e-7, (before_balance_b as f64) * 1e-7);
    let (amt_a, amt_b, lp) = comet_adapter_client.add_liquidity(&token_a_client.address, &token_b_client.address, &amount_a, &amount_b, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[COMET][add_liquidity] Result: amt_a = {:.7}, amt_b = {:.7}, lp = {:.7}", (amt_a as f64) * 1e-7, (amt_b as f64) * 1e-7, (lp as f64) * 1e-7);
    let after_balance_a = token_a_client.balance(user);
    let after_balance_b = token_b_client.balance(user);
    std::println!("[COMET][add_liquidity] User balances after: TKA = {:.7}, TKB = {:.7}", (after_balance_a as f64) * 1e-7, (after_balance_b as f64) * 1e-7);
    let difference_a = before_balance_a - after_balance_a;
    let difference_b = before_balance_b - after_balance_b;
    std::println!("[COMET][add_liquidity] User balance differences: TKA = {:.7}, TKB = {:.7}", (difference_a as f64) * 1e-7, (difference_b as f64) * 1e-7);
    let pool_reserve_a = pool_client.get_balance(&token_a_client.address);
    let pool_reserve_b = pool_client.get_balance(&token_b_client.address);
    let spot_sans_fee_after = pool_client.get_spot_price_sans_fee(&token_a_client.address, &token_b_client.address);
    let implied_price_after = (pool_reserve_a as f64) / (pool_reserve_b as f64);
    std::println!("[COMET][add_liquidity] Pool reserves after: TKA = {:.7}, TKB = {:.7} | Spot price sans fee: {:.7} | Implied price (not considering weight) : {:.7}", (pool_reserve_a as f64) * 1e-7, (pool_reserve_b as f64) * 1e-7, (spot_sans_fee_after as f64) * 1e-7, implied_price_after);

    assert!(amt_a > 0 && amt_b > 0 && lp > 0, "Should add liquidity and receive LP tokens");
    assert!(after_balance_a <= before_balance_a - amt_a, "User TKA balance should decrease by at least amt_a");
    assert!(after_balance_b <= before_balance_b - amt_b, "User TKB balance should decrease by at least amt_b");
    lp
}

pub fn run_remove_liquidity(test_env: &HoopsTestEnvironment, lp_amt: i128) {
    std::println!("\n[COMET][remove_liquidity] Removing liquidity: lp_amt = {:.7}", (lp_amt as f64) * 1e-7);
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let comet_adapter_client = &test_env.adapters.comet;
    let pool = &test_env.comet.pool_ids.get(0).unwrap();
    let slippage_bps: i128 = 500; // 5% slippage
    let comet_pool_client = CometPoolClient::new(env, pool);
    let total_lp = comet_pool_client.get_total_supply();
    let pool_reserve_a = comet_pool_client.get_balance(&token_a_client.address);
    let pool_reserve_b = comet_pool_client.get_balance(&token_b_client.address);
    std::println!("[COMET][remove_liquidity] Pool reserves: TKA = {:.7}, TKB = {:.7}", (pool_reserve_a as f64) * 1e-7, (pool_reserve_b as f64) * 1e-7);
    let spot_no_fee = comet_pool_client.get_spot_price_sans_fee(&token_a_client.address, &token_b_client.address);
    let implied_price = (pool_reserve_a as f64) / (pool_reserve_b as f64);
    std::println!("[COMET][remove_liquidity] Spot price sans fee (A-in-B): {:.7} | Implied price (not considering weight)  (A/B): {:.7}", (spot_no_fee as f64) * 1e-7, implied_price);

    let expected_a = pool_reserve_a.saturating_mul(lp_amt) / total_lp;
    let expected_b = pool_reserve_b.saturating_mul(lp_amt) / total_lp;
    std::println!("[COMET][remove_liquidity] Expected amounts: TKA = {:.7}, TKB = {:.7}", (expected_a as f64) * 1e-7, (expected_b as f64) * 1e-7);

    let amt_a_min: i128 = (expected_a * (10_000 - slippage_bps) / 10_000) as i128;
    let amt_b_min: i128 = (expected_b * (10_000 - slippage_bps) / 10_000) as i128;
    std::println!("[COMET][remove_liquidity] Min amounts: TKA = {:.7}, TKB = {:.7}", (amt_a_min as f64) * 1e-7, (amt_b_min as f64) * 1e-7);
    let deadline = env.ledger().timestamp() + 100;
    std::println!("[COMET][remove_liquidity] Removing liquidity: lp_amt = {:.7} (min out: TKA = {:.7}, TKB = {:.7})", (lp_amt as f64) * 1e-7, (amt_a_min as f64) * 1e-7, (amt_b_min as f64) * 1e-7);
    let (amt_a_out, amt_b_out) = comet_adapter_client.remove_liquidity(pool, &lp_amt, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[COMET][remove_liquidity] Result: amt_a_out = {:.7}, amt_b_out = {:.7}", (amt_a_out as f64) * 1e-7, (amt_b_out as f64) * 1e-7);
    let after_balance_a = token_a_client.balance(user);
    let after_balance_b = token_b_client.balance(user);
      // Log price after remove
    let pool_reserve_a_after = comet_pool_client.get_balance(&token_a_client.address);
    let pool_reserve_b_after = comet_pool_client.get_balance(&token_b_client.address);
    let spot_sans_fee_after = comet_pool_client.get_spot_price_sans_fee(&token_a_client.address, &token_b_client.address);
    let implied_price_after = (pool_reserve_a_after as f64) / (pool_reserve_b_after as f64);
    std::println!("[COMET][remove_liquidity] Pool reserves after: TKA = {:.7}, TKB = {:.7} | Spot price sans fee: {:.7} | Implied price (not considering weight) : {:.7}", (pool_reserve_a_after as f64) * 1e-7, (pool_reserve_b_after as f64) * 1e-7, (spot_sans_fee_after as f64) * 1e-7, implied_price_after);

    std::println!("[COMET][remove_liquidity] User balances after: TKA = {:.7}, TKB = {:.7}", (after_balance_a as f64) * 1e-7, (after_balance_b as f64) * 1e-7);
    assert!(amt_a_out >= amt_a_min, "Should withdraw at least min TKA");
    assert!(amt_b_out >= amt_b_min, "Should withdraw at least min TKB");
    assert!(after_balance_a >= amt_a_out, "User TKA balance should increase by at least amt_a_out");
    assert!(after_balance_b >= amt_b_out, "User TKB balance should increase by at least amt_b_out");
}

pub fn test_comet_adapter(test_env: &HoopsTestEnvironment, failures: i32) -> i32{

    let mut failures = failures;

    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_swap_exact_in(&test_env))) {
        std::println!("[FAIL][COMET][swap_exact_in]: {:?}", e); failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_swap_exact_out(&test_env))) {
        std::println!("[FAIL][COMET][swap_exact_out]: {:?}", e); failures += 1;
    }
    
    // Run add_liquidity and capture the LP token amount
    let lp = match std::panic::catch_unwind(AssertUnwindSafe(|| run_add_liquidity(&test_env))) {
        Ok(lp) => lp,
        Err(e) => {
            std::println!("[FAIL][COMET][add_liquidity]: {:?}", e); failures += 1;
            0 // Default value if the test fails
        }
    };

    // Only run remove_liquidity if add_liquidity succeeded (lp > 0)
    if lp > 0 {
        if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| run_remove_liquidity(&test_env, lp))) {
            std::println!("[FAIL][COMET][remove_liquidity]: {:?}", e); failures += 1;
        }
    } else {
        std::println!("[INFO][COMET] add liquidity failed, skipping remove");
    }
    std::println!("[COMET] Test results: {} failures", failures);
    failures
}

#[test]
pub fn test_comet_adapter_all() {
    let test_env = HoopsTestEnvironment::setup();
    let mut failures = 0;
    failures += test_comet_adapter(&test_env, failures);
    assert_eq!(failures, 0, "Some Comet adapter tests failed");
}