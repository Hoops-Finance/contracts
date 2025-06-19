// Aqua Adapter tests: swap and liquidity
#![cfg(test)]
use soroban_sdk::token;
#[allow(unused_imports)]
use soroban_sdk::vec;
use crate::tests::test_setup::HoopsTestEnvironment;

#[test]
fn test_aqua_adapter_swap_exact_in() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;

    // Mock all authorizations for this test to bypass require_auth errors
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

    token_a_client.approve(&user, &test_env.aqua.router_id.clone().unwrap(), &amount_in, &(env.ledger().timestamp() as u32 + 200));

    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);

    let amount_out = aqua_adapter_client.swap_exact_in(
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
}

#[test]
fn test_aqua_adapter_swap_exact_out() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;
    env.mock_all_auths();
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let aqua_adapter_client = &test_env.adapters.aqua;
    let desired_out: i128 = 500_000;
    let max_in: i128 = 2_000_000;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    token_a_client.approve(&user, &test_env.aqua.router_id.clone().unwrap(), &max_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    let amount_in_used = aqua_adapter_client.swap_exact_out(
        &desired_out,
        &max_in,
        &path,
        user,
        &deadline,
    );
    assert!(amount_in_used <= max_in, "Amount in used should not exceed max_in");
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in_used, "User TKA balance should decrease by amount_in_used");
    assert_eq!(final_user_balance_b, initial_user_balance_b + desired_out, "User TKB balance should increase by desired_out");
}

/*
#[test]
fn test_aqua_adapter_add_and_remove_liquidity() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let aqua_adapter_client = &test_env.adapters.aqua;
    let pool = test_env.aqua.pool_ids.get(0).unwrap();
    let amount_a: i128 = 500_000;
    let amount_b: i128 = 500_000;
    let deadline = env.ledger().timestamp() + 100;
    token_a_client.approve(user, &pool, &amount_a, &(env.ledger().timestamp() as u32 + 200));
    token_b_client.approve(user, &pool, &amount_b, &(env.ledger().timestamp() as u32 + 200));
    let lp_token = aqua_adapter_client.add_liquidity(&token_a_client.address, &token_b_client.address, &amount_a, &amount_b, user, &deadline);
    let (amt_a_out, amt_b_out) = aqua_adapter_client.remove_liquidity(&lp_token, &(amount_a.min(amount_b)), user, &deadline);
    assert!(amt_a_out > 0, "Should withdraw some token A");
    assert!(amt_b_out > 0, "Should withdraw some token B");
}
*/