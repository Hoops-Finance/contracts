#![cfg(test)]

use std::println;
extern crate std;
use crate::c_pool::comet::CometPoolContractClient;
use crate::tests::utils::create_comet_pool;
use sep_41_token::testutils::{MockTokenClient, MockTokenWASM};
use soroban_sdk::String;
use soroban_sdk::{testutils::Address as _, Address};
use soroban_sdk::{vec, Env};

fn create_and_init_token_contract<'a>(
    env: &'a Env,
    admin_id: &'a Address,
    name: &'a str,
    symbol: &'a str,
) -> MockTokenClient<'a> {
    let token_id = env.register_contract_wasm(None, MockTokenWASM);
    let client = MockTokenClient::new(&env, &token_id);
    client.initialize(
        &admin_id,
        &7,
        &String::from_str(&env, name),
        &String::from_str(&env, symbol),
    );
    client
}

fn to_stroop<T: Into<f64>>(a: T) -> i128 {
    (a.into() * 1e7) as i128
}

#[test]
fn test_pool_functions_dep_wdr() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let admin = soroban_sdk::Address::generate(&env);

    // Create Admin
    let admin1 = soroban_sdk::Address::generate(&env);

    // Create 4 tokens
    let token1 = create_and_init_token_contract(&env, &admin1, "SD", "SD");
    let token2 = create_and_init_token_contract(&env, &admin1, "Sample", "RD");

    let user1 = soroban_sdk::Address::generate(&env);
    let user2 = soroban_sdk::Address::generate(&env);
    token1.mint(&admin, &i128::MAX);
    token2.mint(&admin, &i128::MAX);

    token1.mint(&user1, &to_stroop(40000000));
    token1.mint(&user2, &to_stroop(40000000));
    token2.mint(&user1, &to_stroop(40000000));

    let tokens = vec![&env, token1.address.clone(), token2.address.clone()];
    let weights = vec![&env, 5454545, 4545455];
    let balances = vec![&env, to_stroop(50), to_stroop(20)];
    let contract_id =
        create_comet_pool(&env, &admin, &tokens, &weights, &balances, to_stroop(0.003));
    let client = CometPoolContractClient::new(&env, &contract_id);

    token1.approve(&user1, &contract_id, &i128::MAX, &200);
    token2.approve(&user1, &contract_id, &i128::MAX, &200);

    token1.approve(&user2, &contract_id, &i128::MAX, &200);
    token2.approve(&user2, &contract_id, &i128::MAX, &200);

    let pool_supply = client.get_total_supply();
    client.join_pool(&to_stroop(120), &vec![&env, i128::MAX, i128::MAX], &user1);
    assert_eq!(client.get_total_supply(), pool_supply + &to_stroop(120));

    client.exit_pool(&to_stroop(120), &vec![&env, 0, 0], &user1);
    assert_eq!(client.get_total_supply(), pool_supply);

    let total_shares_before_depositing = client.get_total_supply();
    println!("Total Shares before {}", total_shares_before_depositing);
    println!(
        "Total Token Balance before {}",
        client.get_balance(&token2.address)
    );
    println!(
        "Total Token 2 Balance before deposit {}",
        token2.balance(&user1)
    );
    let token_amount_in = client.dep_lp_tokn_amt_out_get_tokn_in(
        &token2.address,
        &to_stroop(0.003),
        &to_stroop(0.005),
        &user1,
    );
    assert_eq!(
        to_stroop(0.003),
        client.get_total_supply() - total_shares_before_depositing
    );

    let total_shares_before_withdrawing = client.get_total_supply();
    println!("Total Shares After {}", total_shares_before_withdrawing);
    println!("Total Amount In {}", token_amount_in);
    println!(
        "Total Token Balance {}",
        client.get_balance(&token2.address)
    );
    println!("Total LP Balance {}", client.balance(&user1));
    println!(
        "Total Token 2 Balance before withdraw {}",
        token2.balance(&user1)
    );

    let token_amount_out = client.wdr_tokn_amt_in_get_lp_tokns_out(
        &token2.address,
        &to_stroop(0.003),
        &to_stroop(0.0001),
        &user1,
    );

    let total_shares = client.get_total_supply();
    println!("Total Shares After {}", total_shares);
    println!("Total Amount In {}", token_amount_out);
    println!(
        "Total Token Balance {}",
        client.get_balance(&token2.address)
    );
    println!(
        "Total Token 2 Balance after withdraw {}",
        token2.balance(&user1)
    );
    assert_eq!(
        total_shares,
        total_shares_before_withdrawing - to_stroop(0.003)
    );

    let prev_token_balance = token1.balance(&user2);

    client.dep_tokn_amt_in_get_lp_tokns_out(
        &token1.address,
        &to_stroop(0.001),
        &to_stroop(0.0001),
        &user2,
    );

    assert_eq!(
        to_stroop(0.001),
        prev_token_balance - token1.balance(&user2)
    );
    let prev_token_balance_before_withdrawing = token1.balance(&user2);

    println!(
        "Prev Token Balance {}",
        prev_token_balance_before_withdrawing
    );
    client.wdr_tokn_amt_out_get_lp_tokns_in(
        &token1.address,
        &to_stroop(0.0009968),
        &to_stroop(0.1),
        &user2,
    );

    println!("Dust Amount {}", client.balance(&user2));
    println!("Token Balance {}", token1.balance(&user2));
    assert_eq!(
        token1.balance(&user2) - prev_token_balance_before_withdrawing,
        to_stroop(0.0009968)
    );
}
