extern crate std;

use phoenix::utils::AdminChange;
use pretty_assertions::assert_eq;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

use crate::{error::ContractError, storage::PENDING_ADMIN, phoenix_stake_tests::setup::deploy_staking_contract};

#[test]
fn propose_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &7u32,
    );

    let result = staking.propose_admin(&new_admin, &None);
    assert_eq!(result, new_admin.clone());

    let pending_admin: AdminChange = env.as_contract(&staking.address, || {
        env.storage().instance().get(&PENDING_ADMIN).unwrap()
    });

    assert_eq!(staking.query_admin(), admin);
    assert_eq!(pending_admin.new_admin, new_admin);
    assert_eq!(pending_admin.time_limit, None);
}

#[test]
fn replace_admin_fails_when_new_admin_is_same_as_current() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &7u32,
    );

    assert_eq!(
        staking.try_propose_admin(&admin, &None),
        Err(Ok(ContractError::SameAdmin))
    );
    assert_eq!(staking.query_admin(), admin);
}

#[test]
fn accept_admin_successfully() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &7u32,
    );

    staking.propose_admin(&new_admin, &None);
    assert_eq!(staking.query_admin(), admin);

    let result = staking.accept_admin();
    assert_eq!(result, new_admin.clone());
    assert_eq!(staking.query_admin(), new_admin);

    let pending_admin: Option<AdminChange> = env.as_contract(&staking.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}

#[test]
fn accept_admin_fails_when_no_pending_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &7u32,
    );

    assert_eq!(
        staking.try_accept_admin(),
        Err(Ok(ContractError::NoAdminChangeInPlace))
    );

    assert_eq!(staking.query_admin(), admin);
}

#[test]
fn accept_admin_fails_when_time_limit_expired() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &7u32,
    );

    let time_limit = 1000u64;
    staking.propose_admin(&new_admin, &Some(time_limit));
    env.ledger().set_timestamp(time_limit + 100);

    assert_eq!(
        staking.try_accept_admin(),
        Err(Ok(ContractError::AdminChangeExpired))
    );
    assert_eq!(staking.query_admin(), admin);
}

#[test]
fn accept_admin_successfully_with_time_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &7u32,
    );

    let time_limit = 1_500;
    staking.propose_admin(&new_admin, &Some(time_limit));
    assert_eq!(staking.query_admin(), admin);

    env.ledger().set_timestamp(1_000u64);

    let result = staking.accept_admin();
    assert_eq!(result, new_admin);
    assert_eq!(staking.query_admin(), new_admin);

    let pending_admin: Option<AdminChange> = env.as_contract(&staking.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}

#[test]
fn accept_admin_successfully_on_time_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &7u32,
    );

    let time_limit = 1_500;
    staking.propose_admin(&new_admin, &Some(time_limit));
    assert_eq!(staking.query_admin(), admin);

    env.ledger().set_timestamp(time_limit);

    let result = staking.accept_admin();
    assert_eq!(result, new_admin);
    assert_eq!(staking.query_admin(), new_admin);

    let pending_admin: Option<AdminChange> = env.as_contract(&staking.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}

#[test]
fn propose_admin_then_revoke() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &7u32,
    );

    staking.propose_admin(&new_admin, &None);
    staking.revoke_admin_change();

    let pending_admin: Option<AdminChange> = env.as_contract(&staking.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });

    assert!(pending_admin.is_none());
}

#[test]
fn revoke_admin_should_fail_when_no_admin_change_in_place() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &7u32,
    );

    assert_eq!(
        staking.try_revoke_admin_change(),
        Err(Ok(ContractError::NoAdminChangeInPlace))
    );
}
