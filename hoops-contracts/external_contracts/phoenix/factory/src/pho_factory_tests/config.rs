use super::setup::{
    deploy_factory_contract, install_latest_factory, install_lp_contract, install_multihop_wasm,
    install_stable_lp, install_stake_wasm, install_token_wasm, lp_contract,
};
use crate::{
    contract::{Factory, FactoryClient},
    error::ContractError,
    storage::ADMIN,
    pho_factory_tests::setup::{generate_lp_init_info, install_and_deploy_token_contract, stable_lp},
};

use test_case::test_case;

use phoenix::utils::PoolType;
use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    vec, Address, BytesN, Env, String, Vec,
};

#[test]
fn test_pho_factory_successfully_inits_itself() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    assert_eq!(factory.get_admin(), admin);
}

#[test]
fn test_pho_factory_successfully_inits_multihop() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let multihop_address = factory.get_config().multihop_address;

    assert!(!multihop_address.to_string().is_empty());
}

#[test]
fn test_pho_factory_successfully_inits_lp() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let mut token1_admin = Address::generate(&env);
    let mut token2_admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        9,
        String::from_str(&env, "Phoenix"),
        String::from_str(&env, "PHO"),
    );
    let mut token2 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        13,
        String::from_str(&env, "Stellar"),
        String::from_str(&env, "XLM"),
    );

    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut token1_admin, &mut token2_admin);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    assert_eq!(factory.get_admin(), admin);

    let lp_init_info = generate_lp_init_info(
        token1.address.clone(),
        token2.address.clone(),
        Address::generate(&env),
        admin.clone(),
        user.clone(),
    );

    factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHO/BTC"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );
    let lp_contract_addr = factory.query_pools().get(0).unwrap();

    let first_lp_contract = lp_contract::Client::new(&env, &lp_contract_addr);
    let share_token_address = first_lp_contract.query_share_token_address();
    let stake_token_address = first_lp_contract.query_stake_contract_address();

    assert_eq!(
        first_lp_contract.query_config(),
        lp_contract::Config {
            fee_recipient: user,
            max_allowed_slippage_bps: 5_000,
            max_allowed_spread_bps: 500,
            max_referral_bps: 5_000,
            pool_type: lp_contract::PairType::Xyk,
            share_token: share_token_address,
            stake_contract: stake_token_address,
            token_a: token1.address,
            token_b: token2.address,
            total_fee_bps: 0,
        }
    );
}

#[test]
fn test_pho_factory_successfully_inits_stable_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = install_and_deploy_token_contract(
        &env,
        token_admin.clone(),
        7,
        String::from_str(&env, "EURO Coin"),
        String::from_str(&env, "EURC"),
    );
    let mut token2 = install_and_deploy_token_contract(
        &env,
        token_admin.clone(),
        7,
        String::from_str(&env, "USD Coin"),
        String::from_str(&env, "USDC"),
    );

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    assert_eq!(factory.get_admin(), admin);

    let lp_init_info = generate_lp_init_info(
        token1.address.clone(),
        token2.address.clone(),
        Address::generate(&env),
        admin.clone(),
        user.clone(),
    );

    factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Pool Stable"),
        &String::from_str(&env, "EURC/USDC"),
        &PoolType::Stable,
        &Some(10),
        &100i64,
        &1_000,
    );

    let lp_contract_addr = factory.query_pools().get(0).unwrap();

    let stable_client = stable_lp::Client::new(&env, &lp_contract_addr);
    let share_token_address = stable_client.query_share_token_address();
    let stake_token_address = stable_client.query_stake_contract_address();

    assert_eq!(
        stable_client.query_config(),
        stable_lp::Config {
            fee_recipient: user,
            max_allowed_slippage_bps: 5_000,
            default_slippage_bps: 2_500,
            max_allowed_spread_bps: 500,
            pool_type: stable_lp::PairType::Stable,
            share_token: share_token_address,
            stake_contract: stake_token_address,
            token_a: token1.address,
            token_b: token2.address,
            total_fee_bps: 0,
        }
    );
}

#[test]
#[should_panic(
    expected = "Factory: Create Liquidity Pool: You are not authorized to create liquidity pool!"
)]
fn test_pho_factory_fails_to_init_lp_when_authorized_address_not_present() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let mut token1_admin = Address::generate(&env);
    let mut token2_admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = Address::generate(&env);
    let mut token2 = Address::generate(&env);

    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    if token2 < token1 {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut token1_admin, &mut token2_admin);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    assert_eq!(factory.get_admin(), admin);

    let lp_init_info = generate_lp_init_info(
        token1.clone(),
        token2.clone(),
        Address::generate(&env),
        admin.clone(),
        user.clone(),
    );

    let unauthorized_addr = Address::generate(&env);

    factory.create_liquidity_pool(
        &unauthorized_addr,
        &lp_init_info,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHO/BTC"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );
}

#[should_panic(
    expected = "Factory: Initialize: there must be at least one whitelisted account able to create liquidity pools."
)]
#[test]
fn test_pho_factory_fails_to_init_lp_when_no_whitelisted_accounts() {
    let env = Env::default();
    let admin = Address::generate(&env);

    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let multihop_wasm_hash = install_multihop_wasm(&env);
    let whitelisted_accounts: Vec<Address> = vec![&env];

    let lp_wasm_hash = install_lp_contract(&env);
    let stable_wasm_hash = install_stable_lp(&env);
    let stake_wasm_hash = install_stake_wasm(&env);
    let token_wasm_hash = install_token_wasm(&env);

    let _ = FactoryClient::new(
        &env,
        &env.register(
            Factory,
            (
                &admin,
                &multihop_wasm_hash,
                &lp_wasm_hash,
                &stable_wasm_hash,
                &stake_wasm_hash,
                &token_wasm_hash,
                whitelisted_accounts,
                &10u32,
            ),
        ),
    );
}

#[test]
fn test_pho_factory_successfully_updates_new_list_of_whitelisted_accounts() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let first_wl_addr = Address::generate(&env);
    let second_wl_addr = Address::generate(&env);

    let factory = deploy_factory_contract(&env, admin.clone());

    let to_add = vec![&env, first_wl_addr.clone(), second_wl_addr.clone()];
    factory.update_config(&None, &None, &None, &None, &Some(to_add), &None, &None);
    // query for first whitelisted address
    let config = factory.get_config();

    assert!(config.whitelisted_accounts.contains(first_wl_addr.clone()));

    let to_remove = vec![&env, admin.clone()];

    factory.update_config(&None, &None, &None, &None, &None, &Some(to_remove), &None);

    let config = factory.get_config();

    assert!(config.whitelisted_accounts.contains(first_wl_addr));
    assert!(config.whitelisted_accounts.contains(second_wl_addr));
    assert!(config.whitelisted_accounts.len() == 2);
}

#[test]
fn test_pho_factory_does_not_change_whitelisted_accounts_when_removing_non_existent() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, admin.clone());

    let to_remove = vec![&env, Address::generate(&env)];

    factory.update_config(&None, &None, &None, &None, &None, &Some(to_remove), &None);

    let config = factory.get_config();

    assert!(config.whitelisted_accounts.contains(admin));
    assert!(config.whitelisted_accounts.len() == 1);
}

#[test]
fn test_pho_factory_add_vec_with_duplicates_should_be_handled_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let first_wl_addr = Address::generate(&env);
    let dupe_of_first_wl_addr = first_wl_addr.clone();
    let second_wl_addr = Address::generate(&env);
    let dupe_second_wl_addr = second_wl_addr.clone();

    let factory = deploy_factory_contract(&env, admin.clone());

    let to_add = vec![
        &env,
        first_wl_addr.clone(),
        dupe_of_first_wl_addr.clone(),
        second_wl_addr.clone(),
        dupe_second_wl_addr.clone(),
    ];

    factory.update_config(&None, &None, &None, &None, &Some(to_add), &None, &None);
    let config = factory.get_config();

    assert!(config.whitelisted_accounts.contains(first_wl_addr.clone()));
    assert!(config.whitelisted_accounts.len() == 3);

    let to_remove = vec![&env, admin.clone()];

    factory.update_config(&None, &None, &None, &None, &None, &Some(to_remove), &None);

    let config = factory.get_config();

    assert!(config.whitelisted_accounts.contains(first_wl_addr));
    assert!(config.whitelisted_accounts.contains(second_wl_addr));
    assert!(config.whitelisted_accounts.len() == 2);
}

#[test]
#[should_panic(expected = "Factory: Create Liquidity Pool: Amp must be set for stable pool")]
fn test_pho_factory_stable_pool_creation_should_fail_early_without_amp() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = install_and_deploy_token_contract(
        &env,
        token_admin.clone(),
        7,
        String::from_str(&env, "EURO Coin"),
        String::from_str(&env, "EURC"),
    );
    let mut token2 = install_and_deploy_token_contract(
        &env,
        token_admin.clone(),
        7,
        String::from_str(&env, "USD Coin"),
        String::from_str(&env, "USDC"),
    );

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    assert_eq!(factory.get_admin(), admin);

    let lp_init_info = generate_lp_init_info(
        token1.address.clone(),
        token2.address.clone(),
        Address::generate(&env),
        admin.clone(),
        user.clone(),
    );

    // we try to make a stable pool without setting the amp
    factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Pool Stable"),
        &String::from_str(&env, "EUROC/USDC"),
        &PoolType::Stable,
        &None,
        &100i64,
        &1_000,
    );
}

#[test]
fn test_pho_factory_create_xyk_pool_with_amp_parameter_should_still_succeed() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = install_and_deploy_token_contract(
        &env,
        token_admin.clone(),
        7,
        String::from_str(&env, "Phoenix"),
        String::from_str(&env, "PHO"),
    );
    let mut token2 = install_and_deploy_token_contract(
        &env,
        token_admin.clone(),
        7,
        String::from_str(&env, "USD Coin"),
        String::from_str(&env, "USDC"),
    );

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    assert_eq!(factory.get_admin(), admin);

    let lp_init_info = generate_lp_init_info(
        token1.address.clone(),
        token2.address.clone(),
        Address::generate(&env),
        admin.clone(),
        user.clone(),
    );

    // we want to make an XYK pool, but we accidentaly set the amp
    // pool creation should still succeed
    factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Pool Stable"),
        &String::from_str(&env, "EUROC/USDC"),
        &PoolType::Xyk,
        &Some(10),
        &100i64,
        &1_000i64,
    );

    let lp_contract_addr = factory.query_pools().get(0).unwrap();

    let first_lp_contract = lp_contract::Client::new(&env, &lp_contract_addr);
    let share_token_address = first_lp_contract.query_share_token_address();
    let stake_token_address = first_lp_contract.query_stake_contract_address();

    assert_eq!(
        first_lp_contract.query_config(),
        lp_contract::Config {
            fee_recipient: user,
            max_allowed_slippage_bps: 5_000,
            max_allowed_spread_bps: 500,
            max_referral_bps: 5_000,
            pool_type: lp_contract::PairType::Xyk,
            share_token: share_token_address,
            stake_contract: stake_token_address,
            token_a: token1.address,
            token_b: token2.address,
            total_fee_bps: 0,
        }
    );
}

#[test]
fn test_pho_factory_update_wasm_hashes() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, admin.clone());
    let old_lp_wasm_hash = factory.get_config().lp_wasm_hash;
    let new_lp_wasm_hash = BytesN::from_array(&env, &[1; 32]);
    assert!(old_lp_wasm_hash != new_lp_wasm_hash);

    factory.update_config(
        &None,
        &Some(new_lp_wasm_hash.clone()),
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    let updated_lp_wasm_hash = factory.get_config().lp_wasm_hash;

    assert!(new_lp_wasm_hash == updated_lp_wasm_hash);
}

#[test]
fn test_pho_factory_migrate_admin_key() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    factory.migrate_admin_key();

    let after_migration: Address = env.as_contract(&factory.address, || {
        env.storage().instance().get(&ADMIN).unwrap()
    });

    assert_eq!(admin, after_migration);
}

#[test]
fn test_pho_factory_update_multihop_addr() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, admin.clone());
    let old_multihop_addr = factory.get_config().multihop_address;
    let new_multihop_adrr = Address::generate(&env);

    assert!(old_multihop_addr != new_multihop_adrr);

    factory.update_config(
        &Some(new_multihop_adrr.clone()),
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    let updated_multihop_addr = factory.get_config().multihop_address;

    assert!(updated_multihop_addr == new_multihop_adrr);
}

#[test]
fn test_pho_factory_simple_update() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let new_wasm_hash = install_latest_factory(&env);
    let latest_wasm = install_stable_lp(&env);

    factory.update(&new_wasm_hash, &latest_wasm);
}

#[test]
fn test_pho_factory_update_lp_token_decimals() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, admin.clone());
    let old_token_decimals = factory.get_config().lp_token_decimals;
    let new_token_decimals = 22;

    assert!(old_token_decimals != new_token_decimals);

    factory.update_config(
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &Some(new_token_decimals),
    );

    let updated_token_decimals = factory.get_config().lp_token_decimals;

    assert!(updated_token_decimals == new_token_decimals);
}

#[test]
fn test_pho_factory_query_version() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let expected_version = env!("CARGO_PKG_VERSION");
    let version = factory.query_version();
    assert_eq!(String::from_str(&env, expected_version), version);
}

#[test_case(0u32; "should fail with 0")]
#[test_case(23u32; "should fail with bigger than 22")]
fn test_pho_factory_update_lp_token_decimals_should_panic_when_invalid(new_token_decimals: u32) {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let factory = deploy_factory_contract(&env, admin.clone());

    assert_eq!(
        factory.try_update_config(
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &Some(new_token_decimals),
        ),
        Err(Ok(ContractError::TokenDecimalsInvalid))
    )
}
