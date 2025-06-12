#![cfg(test)]
#![allow(unused_imports)]

extern crate std;

use crate::{Factory, FactoryClient};
use soroban_sdk::{testutils::Address as _, token::{self, StellarAssetClient}, vec, Address, BytesN, Env};

// The contract that will be deployed by the deployer contract.
mod pool_contract {
    soroban_sdk::contractimport!(file = "../../../bytecodes/comet-pool.wasm");
    pub type PoolClient<'a> = Client<'a>;
}
use pool_contract::PoolClient;

pub fn create_token<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &sac.address()),
        token::StellarAssetClient::new(e, &sac.address()),
    )
}
#[test]
fn test_comet_factory() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let wasm_hash = env.deployer().upload_contract_wasm(pool_contract::WASM);
    // if it's a native test pass the type, if it's a wasm test pass the wasm.
    //let factory_address = env.register(pool_contract::WASM, ());
    let factory_address = env.register(Factory, () );
    let factory_client = FactoryClient::new(&env, &factory_address);
    // Initialize the factory with the contract hash
    factory_client.init(&wasm_hash);

    let controller = Address::generate(&env);
    let (token_1, token_1_admin) = create_token(&env, &controller);
    let (token_2, token_2_admin) = create_token(&env, &controller);

    token_1_admin.mint(&controller, &1_0000000);
    token_2_admin.mint(&controller, &1_0000000);

    let tokens = vec![&env, token_1.address.clone(), token_2.address.clone()];
    let weights = vec![&env, 0_5000000, 0_5000000];
    let balances = vec![&env, 1_0000000, 1_0000000];
    let swap_fee = 0_0030000;

    let salt = BytesN::from_array(&env, &[0; 32]);
    let contract_id =
        factory_client.new_c_pool(&salt, &controller, &tokens, &weights, &balances, &swap_fee);

    let pool_client = PoolClient::new(&env, &contract_id);
    assert_eq!(factory_client.is_c_pool(&contract_id.clone()), true);
    assert_eq!(pool_client.get_controller(), controller);
    assert_eq!(pool_client.get_tokens(), tokens);
    assert_eq!(pool_client.get_swap_fee(), swap_fee);
    assert_eq!(pool_client.get_total_supply(), 100 * 1_0000000);
}
