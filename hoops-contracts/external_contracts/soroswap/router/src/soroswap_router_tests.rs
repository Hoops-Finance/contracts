#![cfg(test)]
#![allow(unused_imports, unused_variables)]
extern crate std;
use crate::{SoroswapRouter, SoroswapRouterClient};
use soroban_sdk::{
    Env, 
    BytesN, 
    Address, 
    testutils::{
        Address as _,
    },
    token::StellarAssetClient
};

// Token Contract
mod token {
    soroban_sdk::contractimport!(file = "../../../bytecodes/soroban_token_contract.wasm");
    pub type TokenClient<'a> = Client<'a>;
}
use token::TokenClient;

pub fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, StellarAssetClient<'a>) {
    let wasm = e.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(e, &wasm.address()),
        StellarAssetClient::new(e, &wasm.address()),
    )
}

// Pair Contract
mod pair {
    soroban_sdk::contractimport!(file = "../../../bytecodes/soroswap_pair.wasm");
   pub type SoroswapPairClient<'a> = Client<'a>;
}
use pair::SoroswapPairClient;


fn pair_contract_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/soroswap_pair.wasm"
    );
    e.deployer().upload_contract_wasm(WASM)
}

// SoroswapFactory Contract
mod factory {
    soroban_sdk::contractimport!(file = "../../../bytecodes/soroswap_factory.wasm");
    pub type SoroswapFactoryClient<'a> = Client<'a>;
}
use factory::SoroswapFactoryClient;

fn create_soroswap_factory<'a>(e: & Env, setter: & Address) -> SoroswapFactoryClient<'a> {
    let pair_hash = pair_contract_wasm(&e);  
    // let factory_address = &e.register_contract_wasm(None, factory::WASM);
    let factory_address = &e.register(factory::WASM, ());
    let factory = SoroswapFactoryClient::new(e, factory_address); 
    factory.initialize(&setter, &pair_hash);
    factory
}

// SoroswapRouter Contract
fn create_soroswap_router<'a>(e: &Env) -> SoroswapRouterClient<'a> {
    SoroswapRouterClient::new(e, &e.register(SoroswapRouter,  () ))
}

// SoroswapRouter TEST

pub struct SoroswapRouterTest<'a> {
    env: Env,
    contract: SoroswapRouterClient<'a>,
    token_0: TokenClient<'a>,
    token_1: TokenClient<'a>,
    factory: SoroswapFactoryClient<'a>,
    user: Address,
    admin: Address
}

impl<'a> SoroswapRouterTest<'a> {
    fn setup() -> Self {

        let env = Env::default();
        env.mock_all_auths();
        let contract = create_soroswap_router(&env);

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        assert_ne!(admin, user);

        let (mut token_0, token_0_admin) = create_token_contract(&env, &admin);
        let (mut token_1, token_1_admin) = create_token_contract(&env, &admin);
        if &token_1.address < &token_0.address {
            std::mem::swap(&mut token_0, &mut token_1);
        }
        token_0.mint(&user, &10_000_000_000_000_000_000);
        token_1.mint(&user, &10_000_000_000_000_000_000);

        let factory = create_soroswap_factory(&env, &admin);
        env.cost_estimate().budget().reset_unlimited();

        SoroswapRouterTest {
            env,
            contract,
            token_0,
            token_1,
            factory,
            user,
            admin
        }
    }

    fn setup_deducted_reserve() -> Self {

        let env = Env::default();
        env.mock_all_auths();
        let contract = create_soroswap_router(&env);

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        assert_ne!(admin, user);

        let (mut token_0, token_0_admin) = create_token_contract(&env, &admin);
        let (mut token_1, token_1_admin) = create_token_contract(&env, &admin);
        if &token_1.address < &token_0.address {
            std::mem::swap(&mut token_0, &mut token_1);
        }
        
        let initial_user_balance = 24_995_705_032_704;

        token_0.mint(&user, &initial_user_balance);
        token_1.mint(&user, &initial_user_balance);

        let factory = create_soroswap_factory(&env, &admin);

        SoroswapRouterTest {
            env,
            contract,
            token_0,
            token_1,
            factory,
            user,
            admin
        }
    }
}

// Test mods:

pub mod initialize;
pub mod add_liquidity;
//pub mod swap;
pub mod remove_liquidity;
pub mod library_functions;
pub mod swap_tokens_for_exact_tokens;
pub mod swap_exact_tokens_for_tokens;
pub mod events;

// BUDGET TEST MOD
mod budget;


