#![cfg(test)]
use soroban_sdk::token;
#[allow(unused_imports)]
use soroban_sdk::{Env, vec};
use crate::tests::test_setup::{ HoopsTestEnvironment};
extern crate std;


#[test]
fn test_soroswap_adapter_swap_exact_in() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;

    // Mock all authorizations for this test to bypass require_auth errors
    env.mock_all_auths();

    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let soroswap_adapter_client = &test_env.adapters.soroswap;

    let amount_in: i128 = 1_000_000; // 1 TKA (assuming 7 decimals)
    let amount_out_min: i128 = 0; // No minimum for this basic test, or a reasonable expectation
    let deadline = env.ledger().timestamp() + 100;

    // Use Vec<Address> for path, matching aggregator example
    let path = vec![&env,
        token_a_client.address.clone(),
        token_b_client.address.clone(),
    ];

    // Approve the router to spend token_a for the user
    token_a_client.approve(&user, &test_env.soroswap.router_id.clone().unwrap(), &amount_in, &(env.ledger().timestamp() as u32 + 200));

    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("Initial user balances: TKA = {}, TKB = {}", initial_user_balance_a, initial_user_balance_b);

    let amount_out = soroswap_adapter_client.swap_exact_in(
        &amount_in,
        &amount_out_min,
        &path,
        user,
        &deadline,
    );
    std::println!("Swap result: amount_in = {}, amount_out = {}", amount_in, amount_out);
    assert!(amount_out > amount_out_min, "Amount out should be greater than min_out");

    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("Final user balances: TKA = {}, TKB = {}", final_user_balance_a, final_user_balance_b);

    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in, "User TKA balance should decrease by amount_in");
    assert!(final_user_balance_b > initial_user_balance_b, "User TKB balance should increase");
    assert_eq!(final_user_balance_b, initial_user_balance_b + amount_out, "User TKB balance should increase by amount_out");
}

#[test]
fn test_soroswap_adapter_swap_exact_out() {
    std::println!("[ADAPTER TESTS] - [SOROSWAP] - [swap_exact_out]");
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let soroswap_adapter_client = &test_env.adapters.soroswap;

    // First, do a swap_exact_in to get a valid amount_out for swap_exact_out
    let amount_in: i128 = 1_000_000;
    let amount_out_min: i128 = 0;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![&env,
        token_a_client.address.clone(),
        token_b_client.address.clone(),
    ];
    token_a_client.approve(&user, &test_env.soroswap.router_id.clone().unwrap(), &amount_in, &(env.ledger().timestamp() as u32 + 200));
    let _ = soroswap_adapter_client.swap_exact_in(
        &amount_in,
        &amount_out_min,
        &path,
        user,
        &deadline,
    );
    // Now, try to swap for a specific output amount (less than what we just got)
    let desired_out: i128 = 500_000; // Should be less than what swap_exact_in would give
    let max_in: i128 = 1_000_000; // Don't allow more than this to be spent
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[ADAPTER TESTS] - [SOROSWAP] - [swap_exact_out]Initial user balances: TKA = {}, TKB = {}", initial_user_balance_a, initial_user_balance_b);
    let amount_in_used = soroswap_adapter_client.swap_exact_out(
        &desired_out,
        &max_in,
        &path,
        user,
        &deadline,
    );
    std::println!("[ADAPTER TESTS] - [SOROSWAP] - [swap_exact_out]SwapExactOut result: max_in = {}, actual_in = {}, desired_out = {}", max_in, amount_in_used, desired_out);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[ADAPTER TESTS] - [SOROSWAP] - [swap_exact_out]Final user balances: TKA = {}, TKB = {}", final_user_balance_a, final_user_balance_b);
    assert!(amount_in_used <= max_in, "Should not use more than max_in");
    assert_eq!(final_user_balance_b, initial_user_balance_b + desired_out, "User TKB balance should increase by desired_out");
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in_used, "User TKA balance should decrease by amount_in_used");
}
/*
#[test]
fn test_soroswap_adapter_add_and_remove_liquidity() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let soroswap_adapter_client = &test_env.adapters.soroswap;
    let pool = test_env.soroswap.pool_ids.get(0).unwrap();
    let pool_client = soroswap_adapter_client.pool_client(&pool);
    let amount_a: i128 = 500_000;
    let amount_b: i128 = 500_000;
    let deadline = env.ledger().timestamp() + 100;
    token_a_client.approve(user, &pool, &amount_a, &(env.ledger().timestamp() as u32 + 200));
    token_b_client.approve(user, &pool, &amount_b, &(env.ledger().timestamp() as u32 + 200));
    let initial_lp_balance = pool.balance(user);
    std::println!("Initial LP token balance: {}", initial_lp_balance);
    let lp_token = soroswap_adapter_client.add_liquidity(&token_a_client.address, &token_b_client.address, &amount_a, &amount_b, user, &deadline);
    let new_lp_balance = pool.balance(user);
    std::println!("LP token after add_liquidity: {}", new_lp_balance);
    let (amt_a_out, amt_b_out) = soroswap_adapter_client.remove_liquidity(&lp_token, &(amount_a.min(amount_b)), user, &deadline);
    let final_lp_balance = pool.balance(user);
    std::println!("LP token after remove_liquidity: {}", final_lp_balance);
    assert!(amt_a_out > 0, "Should withdraw some token A");
    assert!(amt_b_out > 0, "Should withdraw some token B");
}
 */