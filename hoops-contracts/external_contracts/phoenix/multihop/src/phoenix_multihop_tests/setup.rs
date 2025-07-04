use crate::contract::{Multihop, MultihopClient};
use crate::factory_contract::{LiquidityPoolInitInfo, PoolType, StakeInitInfo, TokenInitInfo};
use crate::storage::{DataKey, ADMIN};
use crate::{factory_contract, stable_pool, token_contract, xyk_pool};

use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    Address, BytesN, Env,
};
use soroban_sdk::{vec, String};

#[allow(clippy::too_many_arguments)]
pub mod old_multihop {
    soroban_sdk::contractimport!(file = "../../../bytecodes/old_phoenix_multihop.wasm");
}

#[allow(clippy::too_many_arguments)]
pub mod latest_multihop {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/phoenix_multihop.wasm"
    );
}

pub fn create_token_contract_with_metadata<'a>(
    env: &Env,
    admin: &Address,
    decimals: u32,
    name: String,
    symbol: String,
    amount: i128,
) -> token_contract::Client<'a> {
    let token = token_contract::Client::new(
        env,
        &env.register(token_contract::WASM, (admin, decimals, name, symbol)),
    );
    token.mint(admin, &amount);
    token
}

pub fn install_lp_contract(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(xyk_pool::WASM)
}

pub fn install_stable_lp_contract(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(stable_pool::WASM)
}

pub fn install_token_wasm(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(token_contract::WASM)
}

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn install_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
pub fn install_multihop_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/phoenix_multihop.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn deploy_multihop_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    factory: &Address,
) -> MultihopClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));

    let multihop = MultihopClient::new(env, &env.register(Multihop, (&admin, factory)));

    multihop
}

pub fn deploy_and_mint_tokens<'a>(
    env: &'a Env,
    admin: &'a Address,
    amount: i128,
) -> token_contract::Client<'a> {
    let token = deploy_token_contract(env, admin);
    token.mint(admin, &amount);
    token
}

pub fn deploy_and_initialize_factory(env: &Env, admin: Address) -> factory_contract::Client {
    let multihop_wasm_hash = install_multihop_wasm(env);
    let whitelisted_accounts = vec![env, admin.clone()];

    let lp_wasm_hash = install_lp_contract(env);
    let stable_wasm_hash = install_stable_lp_contract(env);
    let stake_wasm_hash = install_stake_wasm(env);
    let token_wasm_hash = install_token_wasm(env);

    let factory_client = factory_contract::Client::new(
        env,
        &env.register(
            factory_contract::WASM,
            (
                &admin.clone(),
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

    factory_client
}

#[allow(clippy::too_many_arguments)]
pub fn deploy_and_initialize_pool(
    env: &Env,
    factory: &factory_contract::Client,
    admin: Address,
    mut token_a: Address,
    mut token_a_amount: i128,
    mut token_b: Address,
    mut token_b_amount: i128,
    fees: Option<i64>,
    pool_type: PoolType,
) {
    if token_b < token_a {
        std::mem::swap(&mut token_a, &mut token_b);
        std::mem::swap(&mut token_a_amount, &mut token_b_amount);
    }

    let token_init_info = TokenInitInfo {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(env),
        max_complexity: 10u32,
    };

    let lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        max_allowed_slippage_bps: 5000,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 500,
        swap_fee_bps: fees.unwrap_or(0i64),
        max_referral_bps: 5_000,
        token_init_info,
        stake_init_info,
    };

    let amp = match pool_type {
        PoolType::Stable => Some(10u64),
        PoolType::Xyk => None,
    };

    let lp = factory.create_liquidity_pool(
        &admin.clone(),
        &lp_init_info,
        &String::from_str(env, "Pool"),
        &String::from_str(env, "PHO/XLM"),
        &pool_type,
        &amp,
        &100i64,
        &1_000,
    );

    match pool_type {
        PoolType::Xyk => {
            let lp_client = xyk_pool::Client::new(env, &lp);
            lp_client.provide_liquidity(
                &admin.clone(),
                &Some(token_a_amount),
                &None,
                &Some(token_b_amount),
                &None,
                &None::<i64>,
                &None::<u64>,
                &false,
            );
        }
        PoolType::Stable => {
            let lp_client = stable_pool::Client::new(env, &lp);
            lp_client.provide_liquidity(
                &admin.clone(),
                &token_a_amount,
                &token_b_amount,
                &None,
                &None::<u64>,
                &None::<u128>,
                &false,
            );
        }
    }
}

#[test]
#[allow(deprecated)]
#[cfg(feature = "upgrade")]
fn test_pho_multihop_update_multihop() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let factory = Address::generate(&env);

    let old_multhop_addr = env.register_contract_wasm(None, old_multihop::WASM);
    let old_multihop_client = old_multihop::Client::new(&env, &old_multhop_addr);

    old_multihop_client.initialize(&admin, &factory);
    let latest_multihop_wasm = install_multihop_wasm(&env);
    old_multihop_client.update(&latest_multihop_wasm);
}

#[test]
fn test_pho_multihop_migrate_admin_key() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    let before_migration: Address = env.as_contract(&multihop.address, || {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    });

    multihop.migrate_admin_key();

    let after_migration: Address = env.as_contract(&multihop.address, || {
        env.storage().instance().get(&ADMIN).unwrap()
    });

    assert_eq!(before_migration, after_migration);
    assert_ne!(Address::generate(&env), after_migration)
}

#[test]
fn test_pho_multihop_test_update() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let old_multihop_addr = env.register(old_multihop::WASM, ());
    let multihop = old_multihop::Client::new(&env, &old_multihop_addr);
    multihop.initialize(&admin, &Address::generate(&env));

    let new_wasm_hash = install_multihop_wasm(&env);

    multihop.update(&new_wasm_hash);
}

#[test]
fn test_pho_multihop_test_query_version() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    let expected_version = env!("CARGO_PKG_VERSION");
    let version = multihop.query_version();
    assert_eq!(String::from_str(&env, expected_version), version);
}
