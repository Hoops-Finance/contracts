// Tests for add_liquidity and remove_liquidity for all adapters
#![cfg(test)]
use soroban_sdk::token;
use soroban_sdk::{Env, vec};
use crate::tests::test_setup::HoopsTestEnvironment;

#[test]
fn test_adapter_add_and_remove_liquidity() {
    let test_env = HoopsTestEnvironment::setup();
    let env = &test_env.env;
    let user = &test_env.user;
    let token_a_client = token::Client::new(&env, &test_env.tokens.client_a);
    let token_b_client = token::Client::new(&env, &test_env.tokens.client_b);
    let adapters = [&test_env.adapters.soroswap, &test_env.adapters.aqua, &test_env.adapters.phoenix, &test_env.adapters.comet];
    let pools = [
        test_env.soroswap.pool_ids.get(0).unwrap(),
        test_env.aqua.pool_ids.get(0).unwrap(),
        test_env.phoenix.pool_ids.get(0).unwrap(),
        test_env.comet.pool_ids.get(0).unwrap(),
    ];
    let amount_a: i128 = 500_000;
    let amount_b: i128 = 500_000;
    let deadline = env.ledger().timestamp() + 100;
    for (adapter, pool) in adapters.iter().zip(pools.iter()) {
        token_a_client.approve(user, pool, &amount_a, &(env.ledger().timestamp() as u32 + 200));
        token_b_client.approve(user, pool, &amount_b, &(env.ledger().timestamp() as u32 + 200));
        let lp_token = adapter.add_liquidity(&token_a_client.address, &token_b_client.address, &amount_a, &amount_b, user, &deadline);
        // Remove liquidity (try to remove what we just added)
        let (amt_a_out, amt_b_out) = adapter.remove_liquidity(&lp_token, &(amount_a.min(amount_b)), user, &deadline);
        assert!(amt_a_out > 0, "Should withdraw some token A");
        assert!(amt_b_out > 0, "Should withdraw some token B");
    }
}
