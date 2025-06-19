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
    let amount_in: i128 = 1_000_000;
    let amount_out_min: i128 = 0;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    token_a_client.approve(&user, &test_env.soroswap.router_id.clone().unwrap(), &amount_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[SOROSWAP][swap_exact_in] Initial user balances: TKA = {}, TKB = {}", initial_user_balance_a, initial_user_balance_b);
    let amount_out = soroswap_adapter_client.swap_exact_in(&amount_in, &amount_out_min, &path, user, &deadline);
    std::println!("[SOROSWAP][swap_exact_in] Swap result: amount_in = {}, amount_out = {}", amount_in, amount_out);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[SOROSWAP][swap_exact_in] Final user balances: TKA = {}, TKB = {}", final_user_balance_a, final_user_balance_b);
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
    let soroswap_adapter_client = &test_env.adapters.soroswap;
    let desired_out: i128 = 500_000;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    // Calculate required input for desired_out using the router's get_amounts_in
    let router = test_env.soroswap.router_id.clone().unwrap();
    let router_client = crate::tests::test_setup::soroswap_router::SoroswapRouterClient::new(env, &router);
    let amounts_in = router_client.router_get_amounts_in(&desired_out, &path);
    let required_in = amounts_in.get(0).unwrap();
    std::println!("[SOROSWAP][swap_exact_out] Required input for desired_out {} is {}", desired_out, required_in);
    let max_in = required_in; // set max_in to exactly what is needed
    token_a_client.approve(&user, &router, &max_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    std::println!("[SOROSWAP][swap_exact_out] --- BEGIN ---");
    std::println!("[SOROSWAP][swap_exact_out] Params: desired_out = {}, max_in = {}, deadline = {}", desired_out, max_in, deadline);
    std::println!("[SOROSWAP][swap_exact_out] Path: {:?}", path);
    std::println!("[SOROSWAP][swap_exact_out] Initial user balances: TKA = {}, TKB = {}", initial_user_balance_a, initial_user_balance_b);
    let amount_in_used = soroswap_adapter_client.swap_exact_out(&desired_out, &max_in, &path, user, &deadline);
    std::println!("[SOROSWAP][swap_exact_out] SwapExactOut result: max_in = {}, actual_in = {}, desired_out = {}", max_in, amount_in_used, desired_out);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    std::println!("[SOROSWAP][swap_exact_out] Final user balances: TKA = {}, TKB = {}", final_user_balance_a, final_user_balance_b);
    std::println!("[SOROSWAP][swap_exact_out] --- END ---");
    assert!(amount_in_used <= max_in, "Should not use more than max_in");
    assert_eq!(final_user_balance_b, initial_user_balance_b + desired_out, "User TKB balance should increase by desired_out");
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in_used, "User TKA balance should decrease by amount_in_used");
}

pub fn run_add_liquidity(test_env: &HoopsTestEnvironment) {
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let soroswap_adapter_client = &test_env.adapters.soroswap;
    let amount_a: i128 = 500_000;
    let amount_b: i128 = 500_000;
    let amt_a_min: i128 = 1; // Patch: supply min amounts
    let amt_b_min: i128 = 1;
    let deadline = env.ledger().timestamp() + 100;
    let pool = test_env.soroswap.pool_ids.get(0).unwrap();
    let pool_client = crate::tests::test_setup::soroswap_pair::SoroswapPairClient::new(env, &pool);
    let lp_token_client = token::Client::new(env, &pool);
    let before_reserves = pool_client.get_reserves();
    let before_lp_balance = lp_token_client.balance(user);
    std::println!("[SOROSWAP][add_liquidity] Pool reserves before: {:?}", before_reserves);
    std::println!("[SOROSWAP][add_liquidity] User LP balance before: {}", before_lp_balance);
    token_a_client.approve(user, &test_env.soroswap.router_id.clone().unwrap(), &amount_a, &(env.ledger().timestamp() as u32 + 200));
    token_b_client.approve(user, &test_env.soroswap.router_id.clone().unwrap(), &amount_b, &(env.ledger().timestamp() as u32 + 200));
    std::println!("[SOROSWAP][add_liquidity] Adding liquidity: amount_a = {}, amount_b = {}", amount_a, amount_b);
    let (amt_a, amt_b, lp) = soroswap_adapter_client.add_liquidity(&token_a_client.address, &token_b_client.address, &amount_a, &amount_b, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[SOROSWAP][add_liquidity] Result: amt_a = {}, amt_b = {}, lp = {}", amt_a, amt_b, lp);
    let after_reserves = pool_client.get_reserves();
    let after_lp_balance = lp_token_client.balance(user);
    std::println!("[SOROSWAP][add_liquidity] Pool reserves after: {:?}", after_reserves);
    std::println!("[SOROSWAP][add_liquidity] User LP balance after: {}", after_lp_balance);
    assert!(amt_a > 0 && amt_b > 0 && lp > 0, "Should add liquidity and receive LP tokens");
    assert!(after_reserves.0 > before_reserves.0 || after_reserves.1 > before_reserves.1, "Pool reserves should increase");
    assert!(after_lp_balance > before_lp_balance, "User LP balance should increase");
}

pub fn run_remove_liquidity(test_env: &HoopsTestEnvironment) {
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
    std::println!("[SOROSWAP][remove_liquidity] Pool reserves before: {:?}", before_reserves);
    std::println!("[SOROSWAP][remove_liquidity] User LP balance before: {}", before_lp_balance);
    let lp_amt: i128 = 100_000; // Try to remove some LP tokens
    let amt_a_min: i128 = 1; // Patch: supply min amounts
    let amt_b_min: i128 = 1;
    let deadline = env.ledger().timestamp() + 100;
    std::println!("[SOROSWAP][remove_liquidity] Removing liquidity: lp_amt = {}", lp_amt);
    let (amt_a_out, amt_b_out) = soroswap_adapter_client.remove_liquidity(&pool, &lp_amt, &amt_a_min, &amt_b_min, user, &deadline);
    std::println!("[SOROSWAP][remove_liquidity] Result: amt_a_out = {}, amt_b_out = {}", amt_a_out, amt_b_out);
    let after_reserves = pool_client.get_reserves();
    let after_lp_balance = lp_token_client.balance(user);
    std::println!("[SOROSWAP][remove_liquidity] Pool reserves after: {:?}", after_reserves);
    std::println!("[SOROSWAP][remove_liquidity] User LP balance after: {}", after_lp_balance);
    assert!(amt_a_out > 0 || amt_b_out > 0, "Should withdraw some tokens");
    assert!(after_reserves.0 < before_reserves.0 || after_reserves.1 < before_reserves.1, "Pool reserves should decrease");
    assert!(after_lp_balance < before_lp_balance, "User LP balance should decrease");
}

#[test]
pub fn test_soroswap_adapter_all() {
    let test_env = HoopsTestEnvironment::setup();
    run_swap_exact_in(&test_env);
    run_swap_exact_out(&test_env);
    run_add_liquidity(&test_env);
    run_remove_liquidity(&test_env);
}