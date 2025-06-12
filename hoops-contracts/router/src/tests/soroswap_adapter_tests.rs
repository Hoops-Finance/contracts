#![cfg(test)]
use soroban_sdk::token;
#[allow(unused_imports)]
use soroban_sdk::{Env, vec};
use crate::tests::test_setup::{ HoopsTestEnvironment};

#[test]
fn test_soroswap_adapter_swap_exact_in() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;

    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let soroban_token_contract = token::Client::new(&env, &test_env.tokens.client_c);
    let soroswap_adapter_client = &test_env.adapters.soroswap;

    let amount_in: i128 = 1_000_000; // 1 TKA (assuming 7 decimals)
    let amount_out_min: i128 = 0; // No minimum for this basic test, or a reasonable expectation
    let deadline = env.ledger().timestamp() + 100;

    let path = vec![
        env,
        token_a_client.address.clone(),
        token_b_client.address.clone(),
    ];

    // Approve the adapter to spend token_a for the user
    // Note: The adapter itself will transferFrom the user to the Soroswap Router
    // So, the user needs to approve the Soroswap Router, which is done during liquidity add.
    // For swaps, the user calls the adapter, which calls the router.
    // The router pulls tokens from the user. So, user approves router.
    // ensure approvals are sufficient from setup, or add specific approval here if needed.
    // The current setup approves the router for liquidity. For swaps, it also needs approval.

    // Re-approve router just in case, or ensure initial approval is enough
    token_a_client.approve(user, &test_env.soroswap.router_id.clone().unwrap(), &amount_in, &(env.ledger().timestamp() as u32 + 200));


    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);

    let amount_out = soroswap_adapter_client.swap_exact_in(
        &amount_in,
        &amount_out_min,
        &path,
        user,
        &deadline,
    );

    assert!(amount_out > amount_out_min, "Amount out should be greater than min_out");

    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);

    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in, "User TKA balance should decrease by amount_in");
    assert!(final_user_balance_b > initial_user_balance_b, "User TKB balance should increase");
    assert_eq!(final_user_balance_b, initial_user_balance_b + amount_out, "User TKB balance should increase by amount_out");


    // TODO: Add test for swap_exact_out if implemented in the adapter
}
