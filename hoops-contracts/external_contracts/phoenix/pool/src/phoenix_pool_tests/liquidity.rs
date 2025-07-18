extern crate std;

use phoenix::utils::{AutoUnstakeInfo, LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use pretty_assertions::assert_eq;

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger},
    vec, Address, Env, IntoVal, String, Symbol,
};

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::{
    contract::{LiquidityPool, LiquidityPoolClient},
    stake_contract::{
        self, Stake, StakedResponse, WithdrawableReward, WithdrawableRewardsResponse,
    },
    storage::{Asset, PoolResponse},
    phoenix_pool_tests::setup::{install_stake_wasm, install_token_wasm},
    token_contract,
};

#[test]
fn provide_liquidity() {
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
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1_000_000_000_000_000);
    assert_eq!(token1.balance(&user1), 1_000_000_000_000_000);

    token2.mint(&user1, &1_000_000_000_000_000);
    assert_eq!(token2.balance(&user1), 1_000_000_000_000_000);

    pool.provide_liquidity(
        &user1,
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &None,
        &None::<u64>,
        &false,
    );

    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    Symbol::new(&env, "provide_liquidity"),
                    (
                        &user1,
                        Some(1_000_000_000_000_000_i128),
                        Some(1_000_000_000_000_000_i128),
                        Some(1_000_000_000_000_000_i128),
                        Some(1_000_000_000_000_000_i128),
                        None::<i64>,
                        None::<u64>,
                        false,
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token1.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 1_000_000_000_000_000_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token2.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 1_000_000_000_000_000_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },
                ],
            }
        ),]
    );

    assert_eq!(token_share.balance(&user1), 999999999999000);
    assert_eq!(token_share.balance(&pool.address), 1_000);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 1_000_000_000_000_000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 1_000_000_000_000_000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 1_000_000_000_000_000_i128
            },
            asset_b: Asset {
                address: token2.address,
                amount: 1_000_000_000_000_000_i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 1_000_000_000_000_000_i128
            },
            stake_address: result.clone().stake_address,
        }
    );
    assert_eq!(pool.query_total_issued_lp(), 1_000_000_000_000_000);
}

#[test]
fn withdraw_liquidity() {
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
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1_000_000_000_000_000);
    token2.mint(&user1, &1_000_000_000_000_000);
    pool.provide_liquidity(
        &user1,
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &None,
        &None::<u64>,
        &false,
    );

    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token_share.balance(&user1), 999_999_999_999_000);
    assert_eq!(token1.balance(&pool.address), 1_000_000_000_000_000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 1_000_000_000_000_000);

    let share_amount = 500_000_000_000_000;
    let min_a = 500_000_000_000_000;
    let min_b = 500_000_000_000_000;
    pool.withdraw_liquidity(&user1, &share_amount, &min_a, &min_b, &None::<u64>, &None);

    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    Symbol::new(&env, "withdraw_liquidity"),
                    (
                        &user1,
                        500_000_000_000_000_i128,
                        500_000_000_000_000_i128,
                        500_000_000_000_000_i128,
                        None::<u64>,
                        None::<AutoUnstakeInfo>
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        share_token_address.clone(),
                        symbol_short!("transfer"),
                        (&user1, &pool.address, 500_000_000_000_000_i128).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                },],
            }
        ),]
    );

    assert_eq!(token_share.balance(&user1), 499_999_999_999_000);
    assert_eq!(token_share.balance(&pool.address), 1_000); // sanity check
    assert_eq!(token1.balance(&user1), 500_000_000_000_000);
    assert_eq!(token1.balance(&pool.address), 500_000_000_000_000);
    assert_eq!(token2.balance(&user1), 500_000_000_000_000);
    assert_eq!(token2.balance(&pool.address), 500_000_000_000_000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 500_000_000_000_000_i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 500_000_000_000_000_i128,
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 500_000_000_000_000_i128,
            },
            stake_address: result.clone().stake_address,
        }
    );

    // clear the pool
    pool.withdraw_liquidity(
        &user1,
        &499_999_999_999_000, /* leftover shares */
        &499_999_999_999_000,
        &499_999_999_999_000,
        &None::<u64>,
        &None,
    );
    assert_eq!(token_share.balance(&user1), 0);
    assert_eq!(token_share.balance(&pool.address), 1_000); // Because of the minted 1_000 lp shares
    assert_eq!(token1.balance(&user1), 999_999_999_999_000);
    assert_eq!(token2.balance(&user1), 999_999_999_999_000);
    // those tokens are locked because of the initial amount of LP shared tokens that are locked
    // Thanks to that all the subsequent users will be having a proper fair share
    assert_eq!(token1.balance(&pool.address), 1_000);
    assert_eq!(token2.balance(&pool.address), 1_000);
}

#[test]
#[should_panic(
    expected = "Pool: ProvideLiquidity: Both tokens must be provided and must be bigger then 0!"
)]
fn swap_with_no_amounts() {
    let env = Env::default();
    env.mock_all_auths();

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
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    token1.mint(&user1, &10_001_000);
    token2.mint(&user1, &10_001_000);
    // providing all amounts as None
    pool.provide_liquidity(
        &user1,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None::<u64>,
        &false,
    );
}

#[test]
#[should_panic(
    expected = "Pool: WithdrawLiquidity: Minimum amount of token_a or token_b is not satisfied!"
)]
fn withdraw_liqudity_below_min() {
    let env = Env::default();
    env.mock_all_auths();

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
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    token1.mint(&user1, &10_000);
    token2.mint(&user1, &10_000);
    // providing liquidity in a 1:1 ratio
    pool.provide_liquidity(
        &user1,
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &None,
        &None::<u64>,
        &false,
    );

    let share_amount = 5_000;
    // Expecting min_a and/or min_b as huge bigger then available
    pool.withdraw_liquidity(&user1, &share_amount, &30_000, &30_000, &None::<u64>, &None);
}

#[test]
fn query_share_valid_liquidity() {
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
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address.clone());

    token1.mint(&user1, &150_000);
    token1.mint(&user2, &150_000);
    token1.mint(&user3, &600_000);

    token2.mint(&user1, &200_000);
    token2.mint(&user2, &200_000);
    token2.mint(&user3, &800_000);

    // all users provide liquidity in a 3:4 ratio
    pool.provide_liquidity(
        &user1,
        &Some(150_000),
        &Some(150_000),
        &Some(200_000),
        &Some(200_000),
        &None,
        &None::<u64>,
        &false,
    );
    let pool_info_before_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_before_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 150_000
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 200_000
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 173205
            },
            stake_address: pool_info_before_withdrawal.clone().stake_address,
        }
    );
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token2.balance(&user1), 0);

    pool.provide_liquidity(
        &user2,
        &Some(150_000),
        &Some(150_000),
        &Some(200_000),
        &Some(200_000),
        &None,
        &None::<u64>,
        &false,
    );
    let pool_info_before_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_before_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 300_000
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 400_000
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 346_410
            },
            stake_address: pool_info_before_withdrawal.clone().stake_address,
        }
    );
    assert_eq!(token1.balance(&user2), 0);
    assert_eq!(token2.balance(&user2), 0);

    pool.provide_liquidity(
        &user3,
        &Some(600_000),
        &Some(600_000),
        &Some(800_000),
        &Some(800_000),
        &None,
        &None::<u64>,
        &false,
    );
    let pool_info_before_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_before_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 900_000
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 1_200_000
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 1_039_230
            },
            stake_address: pool_info_before_withdrawal.clone().stake_address,
        }
    );
    assert_eq!(token1.balance(&user3), 0);
    assert_eq!(token2.balance(&user3), 0);

    // user1 assertions
    let lp_share_balance_user1 = token_share.balance(&user1);
    let query_share_result_user1 = pool.query_share(&lp_share_balance_user1);
    // first liquidity provided seems to be loosing a bit of tokens due to
    // initially minter 1_000 extra LP share tokens, but this difference
    // will only decrese over time as more people provide plus we are the
    // first provider
    assert_eq!(
        query_share_result_user1,
        (
            Asset {
                address: token1.address.clone(),
                amount: 149_133
            },
            Asset {
                address: token2.address.clone(),
                amount: 198_845
            }
        )
    );

    let lp_share_balance_user2 = token_share.balance(&user2);
    let query_share_result_user2 = pool.query_share(&lp_share_balance_user2);
    assert_eq!(
        query_share_result_user2,
        (
            Asset {
                address: token1.address.clone(),
                amount: 149_999
            },
            Asset {
                address: token2.address.clone(),
                amount: 199_999
            }
        )
    );

    let lp_share_balance_user3 = token_share.balance(&user3);
    let query_share_result_user3 = pool.query_share(&lp_share_balance_user3);
    assert_eq!(
        query_share_result_user3,
        (
            Asset {
                address: token1.address.clone(),
                amount: 599_999
            },
            Asset {
                address: token2.address.clone(),
                amount: 799_999
            }
        )
    );

    pool.withdraw_liquidity(
        &user1,
        &lp_share_balance_user1,
        &149_133i128,
        &198_845i128,
        &None::<u64>,
        &None,
    );
    let pool_info_after_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_after_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 750_867,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 1_001_155,
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 867_025
            },
            stake_address: pool_info_after_withdrawal.clone().stake_address,
        }
    );

    let lp_share_balance_after_withdraw_user1: i128 = token_share.balance(&user1);
    assert_eq!(lp_share_balance_after_withdraw_user1, 0);
    assert_eq!(token1.balance(&user1), 149_133);
    assert_eq!(token2.balance(&user1), 198_845);

    let query_share_result_user1 = pool.query_share(&lp_share_balance_after_withdraw_user1);
    assert_eq!(
        query_share_result_user1,
        (
            Asset {
                address: token1.address.clone(),
                amount: 0
            },
            Asset {
                address: token2.address.clone(),
                amount: 0
            }
        )
    );

    // user2 assertions
    let lp_share_balance_user2 = token_share.balance(&user2);
    let query_share_result_user2 = pool.query_share(&lp_share_balance_user2);
    assert_eq!(
        query_share_result_user2,
        (
            Asset {
                address: token1.address.clone(),
                amount: 150_000
            },
            Asset {
                address: token2.address.clone(),
                amount: 200_000
            }
        )
    );

    pool.withdraw_liquidity(
        &user2,
        &lp_share_balance_user2,
        &150_000i128,
        &200_000i128,
        &None::<u64>,
        &None,
    );
    let pool_info_after_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_after_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 600_867
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 801_155
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 693_820
            },
            stake_address: pool_info_after_withdrawal.clone().stake_address,
        }
    );

    let lp_share_balance_after_withdraw_user2: i128 = token_share.balance(&user2);
    assert_eq!(lp_share_balance_after_withdraw_user2, 0);
    assert_eq!(token1.balance(&user2), 150_000);
    assert_eq!(token2.balance(&user2), 200_000);

    // user3 assertions
    let lp_share_balance_user3 = token_share.balance(&user3);
    let query_share_result_user3 = pool.query_share(&lp_share_balance_user3);
    assert_eq!(
        query_share_result_user3,
        (
            Asset {
                address: token1.address.clone(),
                amount: 600_000
            },
            Asset {
                address: token2.address.clone(),
                amount: 800_000
            }
        )
    );

    // user3 has 693_820 shares, we are withdrawing 93_820
    pool.withdraw_liquidity(&user3, &93_820, &1000i128, &1000i128, &None::<u64>, &None);
    let pool_info_after_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_after_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 519_617
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 692_821
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 600_000
            },
            stake_address: pool_info_after_withdrawal.clone().stake_address,
        }
    );

    let lp_share_balance_after_withdraw_user3: i128 = token_share.balance(&user3);
    assert_eq!(lp_share_balance_after_withdraw_user3, 599_000);

    let query_share_result_user3 = pool.query_share(&lp_share_balance_after_withdraw_user3);
    assert_eq!(
        query_share_result_user3,
        (
            Asset {
                address: token1.address.clone(),
                amount: 518_750
            },
            Asset {
                address: token2.address.clone(),
                amount: 691_666
            }
        )
    );

    // last user cleans up the pool
    pool.withdraw_liquidity(
        &user3,
        &599_000,
        &150_000i128,
        &200_000i128,
        &None::<u64>,
        &None,
    );
    let pool_info_after_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_after_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 867
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 1_155
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 1_000
            },
            stake_address: pool_info_after_withdrawal.clone().stake_address,
        }
    );

    let lp_share_balance_after_withdraw_user2: i128 = token_share.balance(&user3);
    assert_eq!(lp_share_balance_after_withdraw_user2, 0);
    assert_eq!(token1.balance(&user3), 600_000);
    assert_eq!(token2.balance(&user3), 800_000);
}

#[test]
fn query_share_empty_pool() {
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

    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    let lp_share_balance = token_share.balance(&user1);
    let query_share_result = pool.query_share(&lp_share_balance);
    assert_eq!(
        query_share_result,
        (
            Asset {
                address: token1.address,
                amount: 0
            },
            Asset {
                address: token2.address,
                amount: 0
            }
        )
    );
}

#[should_panic(
    expected = "Pool: ProvideLiquidity: Custom slippage tolerance is more than max allowed slippage tolerance"
)]
#[test]
fn provide_liquidity_slippage_tolerance_too_high() {
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

    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
    );

    pool.provide_liquidity(
        &Address::generate(&env),
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &Some(10_001),
        &None::<u64>,
        &false,
    );
}

#[test]
fn test_pho_pool_query_info_for_factory_works() {
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
        stake_manager,
        stake_owner,
    );

    let result = pool.query_pool_info_for_factory();
    // not using result only because we have to take the current contract address, which is not known during the test
    assert_eq!(
        result.pool_response,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 0
            },
            asset_b: Asset {
                address: token2.address,
                amount: 0
            },
            asset_lp_share: Asset {
                address: pool.query_share_token_address(),
                amount: 0
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );
    assert_eq!(result.total_fee_bps, 0);
}

#[test]
fn provide_liqudity_with_deadline_should_work() {
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
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &10_000);
    assert_eq!(token1.balance(&user1), 10_000);

    token2.mint(&user1, &10_000);
    assert_eq!(token2.balance(&user1), 10_000);

    env.ledger().with_mut(|li| li.timestamp = 99);
    pool.provide_liquidity(
        &user1,
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &None,
        &Some(100),
        &false,
    );

    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    Symbol::new(&env, "provide_liquidity"),
                    (
                        &user1,
                        Some(10_000i128),
                        Some(10_000i128),
                        Some(10_000i128),
                        Some(10_000i128),
                        None::<i64>,
                        Some(100u64),
                        false,
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token1.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 10_000_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token2.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 10_000_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },
                ],
            }
        ),]
    );
    assert_eq!(token_share.balance(&user1), 9_000);
    assert_eq!(token_share.balance(&pool.address), 1000);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 10_000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 10_000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 10_000i128
            },
            asset_b: Asset {
                address: token2.address,
                amount: 10_000i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 10_000i128
            },
            stake_address: result.clone().stake_address,
        }
    );
    assert_eq!(pool.query_total_issued_lp(), 10_000);
}

#[test]
#[should_panic(expected = "Pool: Provide Liquidity: Transaction executed after deadline!")]
fn provide_liqudity_past_deadline_should_panic() {
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
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    token1.mint(&user1, &10_000);
    assert_eq!(token1.balance(&user1), 10_000);

    token2.mint(&user1, &10_000);
    assert_eq!(token2.balance(&user1), 10_000);

    env.ledger().with_mut(|li| li.timestamp = 100);
    pool.provide_liquidity(
        &user1,
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &None,
        &Some(99),
        &false,
    );
}

#[test]
fn withdraw_liquidity_with_deadline_should_work() {
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
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &10_000);
    token2.mint(&user1, &10_000);
    pool.provide_liquidity(
        &user1,
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &None,
        &None::<u64>,
        &false,
    );

    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 10_000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 10_000);

    let share_amount = 5_000;
    let min_a = 5_000;
    let min_b = 5_000;
    env.ledger().with_mut(|li| li.timestamp = 49);
    pool.withdraw_liquidity(&user1, &share_amount, &min_a, &min_b, &Some(50), &None);

    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    Symbol::new(&env, "withdraw_liquidity"),
                    (
                        &user1,
                        5_000i128,
                        5_000i128,
                        5_000i128,
                        50u64,
                        None::<AutoUnstakeInfo>
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        share_token_address.clone(),
                        symbol_short!("transfer"),
                        (&user1, &pool.address, 5_000_i128).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                },],
            }
        ),]
    );

    assert_eq!(token_share.balance(&user1), 4_000);
    assert_eq!(token_share.balance(&pool.address), 1_000);
    assert_eq!(token1.balance(&user1), 5_000);
    assert_eq!(token1.balance(&pool.address), 5_000);
    assert_eq!(token2.balance(&user1), 5_000);
    assert_eq!(token2.balance(&pool.address), 5_000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 5_000i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 5_000i128,
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 5_000i128,
            },
            stake_address: result.clone().stake_address,
        }
    );

    env.ledger().with_mut(|li| li.timestamp = 99);
    pool.withdraw_liquidity(&user1, &4_000, &4_000, &4_000, &Some(100), &None);
    assert_eq!(token_share.balance(&user1), 0);
    assert_eq!(token_share.balance(&pool.address), 1_000); // sanity check
    assert_eq!(token1.balance(&user1), 9_000);
    assert_eq!(token1.balance(&pool.address), 1_000);
    assert_eq!(token2.balance(&user1), 9_000);
    assert_eq!(token2.balance(&pool.address), 1_000);
}

#[test]
#[should_panic(expected = "Pool: Withdraw Liquidity: Transaction executed after deadline!")]
fn withdraw_liquidity_past_deadline_should_panic() {
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
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    token1.mint(&user1, &10_000);
    token2.mint(&user1, &10_000);
    pool.provide_liquidity(
        &user1,
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &Some(10_000),
        &None,
        &None::<u64>,
        &false,
    );

    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 10_000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 10_000);

    let share_amount = 5_000;
    let min_a = 5_000;
    let min_b = 5_000;
    env.ledger().with_mut(|li| li.timestamp = 50);
    pool.withdraw_liquidity(&user1, &share_amount, &min_a, &min_b, &Some(49), &None);
}

#[test]
fn provide_liquidity_and_autostake() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    let reward_token = deploy_token_contract(&env, &admin2);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0,
        None,
        None,
        None,
        stake_manager.clone(),
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    let result = pool.query_pool_info();

    let stake = stake_contract::Client::new(&env, &result.stake_address);

    stake.create_distribution_flow(&stake_manager, &reward_token.address);

    let reward_amount = 1_000_000_000_000_000;
    //100_000_000
    reward_token.mint(&stake_manager, &reward_amount);
    let distribution_duration = 10_000;
    stake.fund_distribution(
        &stake_manager,
        &0,
        &distribution_duration,
        &reward_token.address,
        &reward_amount,
    );

    token1.mint(&user, &1_000_000_000_000_000);
    assert_eq!(token1.balance(&user), 1_000_000_000_000_000);

    token2.mint(&user, &1_000_000_000_000_000);
    assert_eq!(token2.balance(&user), 1_000_000_000_000_000);

    let stake_timestamp = env.ledger().timestamp();
    pool.provide_liquidity(
        &user,
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &None,
        &None::<u64>,
        &true,
    );

    assert_eq!(token_share.balance(&user), 0); // because lp_shares are automatically staked
    assert_eq!(token_share.balance(&pool.address), 1_000);
    assert_eq!(token1.balance(&user), 0);
    assert_eq!(token1.balance(&pool.address), 1_000_000_000_000_000);
    assert_eq!(token2.balance(&user), 0);
    assert_eq!(token2.balance(&pool.address), 1_000_000_000_000_000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 1_000_000_000_000_000_i128
            },
            asset_b: Asset {
                address: token2.address,
                amount: 1_000_000_000_000_000_i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 1_000_000_000_000_000_i128
            },
            stake_address: result.clone().stake_address,
        }
    );
    assert_eq!(pool.query_total_issued_lp(), 1_000_000_000_000_000);

    let stake_client = stake_contract::Client::new(&env, &result.stake_address);
    assert_eq!(
        stake_client.query_staked(&user),
        StakedResponse {
            stakes: vec![
                &env,
                Stake {
                    stake: 999_999_999_999_000, // the og lp_share amount is already staked
                    stake_timestamp
                }
            ]
        }
    );

    assert_eq!(
        stake.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 0
                }
            ]
        }
    );

    env.ledger().set_timestamp(distribution_duration / 2);
    stake.distribute_rewards();

    // og stake amount is 999_999_999_999_000 rewards should be 1/2 of that
    assert_eq!(
        stake.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address,
                    reward_amount: 499999999999965
                }
            ]
        }
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #503)")]
fn provide_liquidity_and_autostake_should_fail_when_liquidity_is_not_enough_for_min_stake() {
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
    let user = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let token_init_info = TokenInitInfo {
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 1_000_000_000_000_000,
        min_reward: 1_000_000_000_000_000,
        manager: stake_manager,
        max_complexity: 10u32,
    };
    let stake_wasm_hash = install_stake_wasm(&env);
    let token_wasm_hash = install_token_wasm(&env);

    let lp_init_info = LiquidityPoolInitInfo {
        admin: admin1,
        swap_fee_bps: 100,
        fee_recipient: Address::generate(&env),
        max_allowed_slippage_bps: 5_000,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 1_000,
        max_referral_bps: 5_000,
        token_init_info,
        stake_init_info,
    };

    let pool = LiquidityPoolClient::new(
        &env,
        &env.register(
            LiquidityPool,
            (
                &stake_wasm_hash,
                &token_wasm_hash,
                lp_init_info,
                &stake_owner,
                String::from_str(&env, "Pool"),
                String::from_str(&env, "PHOBTC"),
                &100i64,
                &1_000i64,
            ),
        ),
    );

    token1.mint(&user, &1_000_000_000_000_000);
    assert_eq!(token1.balance(&user), 1_000_000_000_000_000);

    token2.mint(&user, &1_000_000_000_000_000);
    assert_eq!(token2.balance(&user), 1_000_000_000_000_000);

    pool.provide_liquidity(
        &user,
        &Some(1_000_000_000),
        &Some(1_000_000_000_000),
        &Some(1_000_000_000),
        &Some(1_000_000_000),
        &None,
        &None::<u64>,
        &true,
    );
}

#[test]
fn withdraw_liquidity_with_auto_unstake() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let user = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0,
        None,
        None,
        None,
        stake_manager.clone(),
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    let result = pool.query_pool_info();
    let stake = stake_contract::Client::new(&env, &result.stake_address);

    stake.create_distribution_flow(&stake_manager, &reward_token.address);
    let reward_amount = 1_000_000_000_000_000;
    reward_token.mint(&stake_manager, &reward_amount);
    let distribution_duration = 10_000;
    stake.fund_distribution(
        &stake_manager,
        &0,
        &distribution_duration,
        &reward_token.address,
        &reward_amount,
    );

    token1.mint(&user, &1_000_000_000_000_000);
    token2.mint(&user, &1_000_000_000_000_000);
    let stake_timestamp = env.ledger().timestamp();
    pool.provide_liquidity(
        &user,
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &None,
        &None::<u64>,
        &true,
    );

    let initial_stake = stake.query_staked(&user);
    assert_eq!(
        initial_stake,
        StakedResponse {
            stakes: vec![
                &env,
                Stake {
                    stake: 999_999_999_999_000,
                    stake_timestamp
                }
            ]
        }
    );
    assert_eq!(
        initial_stake.stakes.get(0).unwrap().stake,
        999_999_999_999_000
    );
    assert_eq!(
        initial_stake.stakes.get(0).unwrap().stake_timestamp,
        stake_timestamp
    );

    let share_amount = 500_000_000_000_000;
    let min_a = 500_000_000_000_000;
    let min_b = 500_000_000_000_000;
    let auto_unstake_info = AutoUnstakeInfo {
        stake_amount: initial_stake.stakes.get(0).unwrap().stake,
        stake_timestamp,
    };

    env.ledger().with_mut(|li| li.timestamp += 1000);
    stake.distribute_rewards();
    assert_eq!(
        stake.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 99_999_999_999_900
                }
            ]
        }
    );

    assert_eq!(reward_token.balance(&user), 0); // should be 0 as the user has no rewards so far
    pool.withdraw_liquidity(
        &user,
        &share_amount,
        &min_a,
        &min_b,
        &None::<u64>,
        &Some(auto_unstake_info),
    );

    assert_eq!(reward_token.balance(&user), 99_999_999_999_900); // all rewards withdrawn

    let remaining_stake = stake.query_staked(&user);
    assert!(remaining_stake.stakes.is_empty());
    assert!(remaining_stake.stakes.get(0).is_none(),);

    assert_eq!(token1.balance(&user), 500_000_000_000_000);
    assert_eq!(token2.balance(&user), 500_000_000_000_000);
    assert_eq!(token_share.balance(&user), 499999999999000); // should be 1/2 of the og staked

    env.ledger().with_mut(|li| li.timestamp += 1000);
    stake.distribute_rewards();
    assert_eq!(
        stake.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address,
                    reward_amount: 0
                }
            ]
        }
    );

    pool.withdraw_liquidity(
        &user,
        &499_999_999_999_000,
        &499_999_999_999_000,
        &499_999_999_999_000,
        &None::<u64>,
        &None,
    );

    let final_stake = stake.query_staked(&user);
    assert!(final_stake.stakes.is_empty());

    assert_eq!(token1.balance(&user), 999_999_999_999_000);
    assert_eq!(token2.balance(&user), 999_999_999_999_000);
    assert_eq!(token_share.balance(&user), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #509)")]
fn withdraw_liquidity_with_auto_unstake_should_fail_when_unexisting_stake() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let user = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0,
        None,
        None,
        None,
        stake_manager.clone(),
        stake_owner,
    );

    let share_amount = 500_000_000_000_000;
    let min_a = 500_000_000_000_000;
    let min_b = 500_000_000_000_000;
    let auto_unstake_info = AutoUnstakeInfo {
        stake_amount: 999_999_999_999_000,
        stake_timestamp: 0,
    };

    pool.withdraw_liquidity(
        &user,
        &share_amount,
        &min_a,
        &min_b,
        &None::<u64>,
        &Some(auto_unstake_info),
    );
}
