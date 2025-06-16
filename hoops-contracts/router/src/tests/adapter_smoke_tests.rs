// Tests for Aqua, Phoenix, and Comet adapters using the HoopsTestEnvironment
#![cfg(test)]
use soroban_sdk::token;
use soroban_sdk::{Env, vec};
use crate::tests::test_setup::HoopsTestEnvironment;

#[test]
fn test_aqua_adapter_swap_exact_in() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let aqua_adapter_client = &test_env.adapters.aqua;
    let amount_in: i128 = 1_000_000;
    let amount_out_min: i128 = 0;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    token_a_client.approve(user, &test_env.aqua.router_id.clone().unwrap(), &amount_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    let amount_out = aqua_adapter_client.swap_exact_in(&amount_in, &amount_out_min, &path, user, &deadline);
    assert!(amount_out > amount_out_min);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in);
    assert!(final_user_balance_b > initial_user_balance_b);
    assert_eq!(final_user_balance_b, initial_user_balance_b + amount_out);
}

#[test]
fn test_phoenix_adapter_swap_exact_in() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let phoenix_adapter_client = &test_env.adapters.phoenix;
    let amount_in: i128 = 1_000_000;
    let amount_out_min: i128 = 0;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    token_a_client.approve(user, &test_env.phoenix.pool_ids.get(0).unwrap(), &amount_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    let amount_out = phoenix_adapter_client.swap_exact_in(&amount_in, &amount_out_min, &path, user, &deadline);
    assert!(amount_out > amount_out_min);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in);
    assert!(final_user_balance_b > initial_user_balance_b);
    assert_eq!(final_user_balance_b, initial_user_balance_b + amount_out);
}

#[test]
fn test_comet_adapter_swap_exact_in() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let comet_adapter_client = &test_env.adapters.comet;
    let amount_in: i128 = 1_000_000;
    let amount_out_min: i128 = 0;
    let deadline = env.ledger().timestamp() + 100;
    let path = vec![env, token_a_client.address.clone(), token_b_client.address.clone()];
    token_a_client.approve(user, &test_env.comet.pool_ids.get(0).unwrap(), &amount_in, &(env.ledger().timestamp() as u32 + 200));
    let initial_user_balance_a = token_a_client.balance(user);
    let initial_user_balance_b = token_b_client.balance(user);
    let amount_out = comet_adapter_client.swap_exact_in(&amount_in, &amount_out_min, &path, user, &deadline);
    assert!(amount_out > amount_out_min);
    let final_user_balance_a = token_a_client.balance(user);
    let final_user_balance_b = token_b_client.balance(user);
    assert_eq!(final_user_balance_a, initial_user_balance_a - amount_in);
    assert!(final_user_balance_b > initial_user_balance_b);
    assert_eq!(final_user_balance_b, initial_user_balance_b + amount_out);
}
