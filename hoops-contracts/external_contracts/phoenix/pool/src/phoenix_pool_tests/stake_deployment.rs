extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Env};

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::{
    stake_contract,
    storage::{Config, PairType},
};

#[test]
fn test_pho_pool_confirm_stake_contract_deployment() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1.clone(),
        500,
        200,
        stake_manager.clone(),
        stake_owner.clone(),
    );

    let share_token_address = pool.query_share_token_address();
    let stake_token_address = pool.query_stake_contract_address();

    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address.clone(),
            share_token: share_token_address.clone(),
            stake_contract: stake_token_address.clone(),
            pool_type: PairType::Xyk,
            total_fee_bps: 0,
            fee_recipient: user1,
            max_allowed_slippage_bps: 500,
            max_allowed_spread_bps: 200,
            max_referral_bps: 5_000,
        }
    );

    let stake_client = stake_contract::Client::new(&env, &stake_token_address);
    assert_eq!(
        stake_client.query_config(),
        stake_contract::ConfigResponse {
            config: stake_contract::Config {
                lp_token: share_token_address,
                min_bond: 10,
                min_reward: 5,
                manager: stake_manager,
                owner: stake_owner,
                max_complexity: 10,
            }
        }
    );
}
