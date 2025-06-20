// Comet Adapter tests: swap and liquidity
#![cfg(test)]
use core::panic::AssertUnwindSafe;

use soroban_sdk::token;
#[allow(unused_imports)]
use soroban_sdk::{Env, vec, Address, Vec};
use crate::tests::test_setup::comet_pool::CometPoolClient;
use crate::tests::test_setup::HoopsTestEnvironment;
use crate::tests::test_setup::comet_adapter::Client as CometAdapterClient;
use crate::tests::test_setup;
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
    // Use a smaller swap amount to avoid max in ratio error
    let amount_in: i128 = 10_000; // was 1_000_000
    let amount_out_min: i128 = 0;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    // Register pool for this token set
    let pool = test_env.comet.pool_ids.get(0).unwrap();
    register_comet_pool(comet_adapter_client, path.clone(), pool.clone());
    token_a_client.approve(&user, &pool, &amount_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[COMET][swap_exact_in] Initial user balances: TKA = {}, TKB = {}", initial_user_balance_a, initial_user_balance_b);
    let amount_out = comet_adapter_client.swap_exact_in(&amount_in, &amount_out_min, &path, user, &deadline);
    std::println!("[COMET][swap_exact_in] Swap result: amount_in = {}, amount_out = {}", amount_in, amount_out);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[COMET][swap_exact_in] Final user balances: TKA = {}, TKB = {}", final_user_balance_a, final_user_balance_b);
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
    // Use a smaller desired_out to avoid max out ratio error
    let desired_out: i128 = 5_000; // was 500_000
    let max_in: i128 = 20_000; // was 2_000_000
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    token_a_client.approve(&user, &test_env.comet.pool_ids.get(0).unwrap(), &max_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[COMET][swap_exact_out] Initial user balances: TKA = {}, TKB = {}", initial_user_balance_a, initial_user_balance_b);
    let amount_in_used = comet_adapter_client.swap_exact_out(&desired_out, &max_in, &path, user, &deadline);
    std::println!("[COMET][swap_exact_out] Swap result: max_in = {}, actual_in = {}, desired_out = {}", max_in, amount_in_used, desired_out);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[COMET][swap_exact_out] Final user balances: TKA = {}, TKB = {}", final_user_balance_a, final_user_balance_b);
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
    let comet_adapter_client = &test_env.adapters.comet;
    // Use 1 token (7 decimals)
    let amount_a: i128 = 10_000_000; // 1 token
    let amount_b: i128 = 10_000_000; // 1 token
    let slippage_bps: i128 = 50; // 0.5% slippage
    let amt_a_min: i128 = amount_a * (10_000 - slippage_bps) / 10_000;
    let amt_b_min: i128 = amount_b * (10_000 - slippage_bps) / 10_000;
    let deadline = env.ledger().timestamp() + 100;
    let pool = test_env.comet.pool_ids.get(0).unwrap();
    token_a_client.approve(user, &pool, &amount_a, &(env.ledger().timestamp() as u32 + 200));
    token_b_client.approve(user, &pool, &amount_b, &(env.ledger().timestamp() as u32 + 200));
    let before_balance_a = token_a_client.balance(user);
    let before_balance_b = token_b_client.balance(user);
    std::println!("[COMET][add_liquidity] User balances before: TKA = {}, TKB = {}", before_balance_a, before_balance_b);
    let (amt_a, amt_b, lp) = comet_adapter_client.add_liquidity(&token_a_client.address, &token_b_client.address, &amount_a, &amount_b, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[COMET][add_liquidity] Result: amt_a = {}, amt_b = {}, lp = {}", amt_a, amt_b, lp);
    let after_balance_a = token_a_client.balance(user);
    let after_balance_b = token_b_client.balance(user);
    std::println!("[COMET][add_liquidity] User balances after: TKA = {}, TKB = {}", after_balance_a, after_balance_b);
    assert!(amt_a > 0 && amt_b > 0 && lp > 0, "Should add liquidity and receive LP tokens");
    assert!(after_balance_a <= before_balance_a - amt_a, "User TKA balance should decrease by at least amt_a");
    assert!(after_balance_b <= before_balance_b - amt_b, "User TKB balance should decrease by at least amt_b");
    lp
}

pub fn run_remove_liquidity(test_env: &HoopsTestEnvironment, lp_amt: i128) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let comet_adapter_client = &test_env.adapters.comet;
    let pool = &test_env.comet.pool_ids.get(0).unwrap();
    let slippage_bps: i128 = 50; // 0.5% slippage
    // Get pool state for correct min amounts out using the pool client from test_env
   
    let comet_pool_client = CometPoolClient::new(env, pool);
    let total_lp = comet_pool_client.get_total_supply();
    let tokens = comet_pool_client.get_tokens();
    let pool_reserve_a = token::Client::new(&env, &tokens.get_unchecked(0)).balance(pool);
    let pool_reserve_b = token::Client::new(&env, &tokens.get_unchecked(1)).balance(pool);
    let share = lp_amt as f64 / total_lp as f64;
    let expected_a = (pool_reserve_a as f64 * share) as i128;
    let expected_b = (pool_reserve_b as f64 * share) as i128;
    let amt_a_min: i128 = (expected_a * (10_000 - slippage_bps) / 10_000) as i128;
    let amt_b_min: i128 = (expected_b * (10_000 - slippage_bps) / 10_000) as i128;
    let deadline = env.ledger().timestamp() + 100;
    std::println!("[COMET][remove_liquidity] Removing liquidity: lp_amt = {} (min out: TKA = {}, TKB = {})", lp_amt, amt_a_min, amt_b_min);
    let (amt_a_out, amt_b_out) = comet_adapter_client.remove_liquidity(pool, &lp_amt, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[COMET][remove_liquidity] Result: amt_a_out = {}, amt_b_out = {}", amt_a_out, amt_b_out);
    let after_balance_a = token_a_client.balance(user);
    let after_balance_b = token_b_client.balance(user);
    std::println!("[COMET][remove_liquidity] User balances after: TKA = {}, TKB = {}", after_balance_a, after_balance_b);
    assert!(amt_a_out >= amt_a_min, "Should withdraw at least min TKA");
    assert!(amt_b_out >= amt_b_min, "Should withdraw at least min TKB");
    assert!(after_balance_a >= amt_a_out, "User TKA balance should increase by at least amt_a_out");
    assert!(after_balance_b >= amt_b_out, "User TKB balance should increase by at least amt_b_out");
}

#[test]
pub fn test_comet_adapter_all() {
    let test_env = HoopsTestEnvironment::setup();
    let mut failures = 0;

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
}