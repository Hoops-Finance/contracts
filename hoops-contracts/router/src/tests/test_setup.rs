#![cfg(test)]
#![allow(unused_imports, unused_variables, dead_code)]
use crate::tests::setuputils::generate_pho_lp_init_info;
use hoops_common::types::{PhoLiquidityPoolInitInfo, PhoenixPoolType};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger, LedgerInfo}, // Ledger, LedgerInfo, Events might be unused now, but keep for now
    token,
    vec,
    Address,
    BytesN,
    Env,
    IntoVal,
    String,
    Vec,
};
use aqua_token_share::token_contract::{Client as ShareTokenClient, WASM as SHARE_WASM};
use aqua_soroban_liquidity_pool_router_contract::testutils::{create_reward_boost_feed_contract, create_plane_contract};

//testutils::create_reward_boost_feed_contract;

extern crate std;
// WASM files from the bytecodes directory
const TOKEN_WASM: &[u8] = include_bytes!("../../../bytecodes/soroban_token_contract.wasm");
const SOROSWAP_ROUTER_WASM: &[u8] = include_bytes!("../../../bytecodes/soroswap_router.wasm");
const SOROSWAP_FACTORY_WASM: &[u8] = include_bytes!("../../../bytecodes/soroswap_factory.wasm");
const SOROSWAP_PAIR_WASM: &[u8] = include_bytes!("../../../bytecodes/soroswap_pair.wasm"); // Needed for direct interaction or if router needs it
const AQUA_ROUTER_WASM: &[u8] =
    include_bytes!("../../../bytecodes/aqua_liquidity_pool_router_contract.wasm"); // Assuming this is the main entry for Aqua
const AQUA_POOL_CONSTANT_WASM: &[u8] =
    include_bytes!("../../../bytecodes/aqua_liquidity_pool_contract.wasm");
const AQUA_STABLE_POOL_WASM: &[u8] =
    include_bytes!("../../../bytecodes/aqua_liquidity_pool_stableswap_contract.wasm");

const PHOENIX_FACTORY_WASM: &[u8] = include_bytes!("../../../bytecodes/phoenix_factory.wasm");
const PHOENIX_POOL_WASM: &[u8] = include_bytes!("../../../bytecodes/phoenix_pool.wasm");
const PHOENIX_POOL_STABLE_WASM: &[u8] =
    include_bytes!("../../../bytecodes/phoenix_pool_stable.wasm");
const PHOENIX_MULTIHOP_WASM: &[u8] = include_bytes!("../../../bytecodes/phoenix_multihop.wasm");
const PHOENIX_STAKE_WASM: &[u8] = include_bytes!("../../../bytecodes/phoenix_stake.wasm");
const PHOENIX_TOKEN_WASM: &[u8] =
    include_bytes!("../../../bytecodes/soroban_token_contract_phoenix.wasm");

const COMET_FACTORY_WASM: &[u8] = include_bytes!("../../../bytecodes/comet_factory.wasm");
const COMET_POOL_WASM: &[u8] = include_bytes!("../../../bytecodes/comet_pool.wasm"); // Added

const SOROSWAP_ADAPTER_WASM: &[u8] = include_bytes!("../../../bytecodes/soroswap_adapter.wasm");
const AQUA_ADAPTER_WASM: &[u8] = include_bytes!("../../../bytecodes/aqua_adapter.wasm");
const PHOENIX_ADAPTER_WASM: &[u8] = include_bytes!("../../../bytecodes/phoenix_adapter.wasm");
const COMET_ADAPTER_WASM: &[u8] = include_bytes!("../../../bytecodes/comet_adapter.wasm");

const HOOPS_ROUTER_WASM: &[u8] = include_bytes!("../../../bytecodes/hoops_router.wasm");

// Import clients for deployed contracts
// Standard Token Client from soroswap_token_contract
pub mod soroban_token_contract {
    soroban_sdk::contractimport!(file = "../bytecodes/soroban_token_contract.wasm");
    pub type TokenClient<'a> = Client<'a>;
}
use soroban_token_contract::TokenClient;

// Soroswap Clients
pub mod soroswap_router {
    soroban_sdk::contractimport!(file = "../bytecodes/soroswap_router.wasm");
    pub type SoroswapRouterClient<'a> = Client<'a>;
}
use soroswap_router::SoroswapRouterClient;

pub mod soroswap_factory {
    soroban_sdk::contractimport!(file = "../bytecodes/soroswap_factory.wasm");
    pub type SoroswapFactoryClient<'a> = Client<'a>;
}
use soroswap_factory::SoroswapFactoryClient;
pub mod soroswap_pair {
    soroban_sdk::contractimport!(file = "../bytecodes/soroswap_pair.wasm");
    pub type SoroswapPairClient<'a> = Client<'a>;
}
use soroswap_pair::SoroswapPairClient;

// Aqua Clients
pub mod aqua_router {
    soroban_sdk::contractimport!(file = "../bytecodes/aqua_liquidity_pool_router_contract.wasm");
    pub type AquaRouterClient<'a> = Client<'a>;
}
use aqua_router::AquaRouterClient;
pub mod aqua_pool_constant {
    soroban_sdk::contractimport!(file = "../bytecodes/aqua_liquidity_pool_contract.wasm");
    pub type AquaPoolClient<'a> = Client<'a>;
}
use aqua_pool_constant::AquaPoolClient;
pub mod aqua_stable_pool {
    soroban_sdk::contractimport!(
        file = "../bytecodes/aqua_liquidity_pool_stableswap_contract.wasm"
    );
    pub type AquaStablePoolClient<'a> = Client<'a>;
}
use aqua_stable_pool::AquaStablePoolClient;
pub mod aqua_liquidity_pool_plane {
    soroban_sdk::contractimport!(file = "../bytecodes/aqua_liquidity_pool_plane_contract.wasm");
    pub type AquaLiquidityPoolPlaneClient<'a> = Client<'a>;
}
use aqua_liquidity_pool_plane::AquaLiquidityPoolPlaneClient;

//Phoenix Clients
pub mod phoenix_factory {
    soroban_sdk::contractimport!(file = "../bytecodes/phoenix_factory.wasm");
    pub type PhoenixFactoryClient<'a> = Client<'a>;
}
use phoenix_factory::PhoenixFactoryClient;

pub mod phoenix_pool {
    soroban_sdk::contractimport!(file = "../bytecodes/phoenix_pool.wasm");
    pub type PhoenixPoolClient<'a> = Client<'a>;
}
pub use phoenix_pool::PhoenixPoolClient;


pub mod phoenix_pool_stable {
    soroban_sdk::contractimport!(file = "../bytecodes/phoenix_pool_stable.wasm");
    pub type PhoenixPoolStableClient<'a> = Client<'a>;
}
pub use phoenix_pool_stable::PhoenixPoolStableClient;

pub mod phoenix_multihop {
    soroban_sdk::contractimport!(file = "../bytecodes/phoenix_multihop.wasm");
    pub type PhoenixMultihopClient<'a> = Client<'a>;
}
use phoenix_multihop::PhoenixMultihopClient;
pub mod phoenix_stake {
    soroban_sdk::contractimport!(file = "../bytecodes/phoenix_stake.wasm");
    pub type PhoenixStakeClient<'a> = Client<'a>;
}
use phoenix_stake::PhoenixStakeClient;
pub mod phoenix_token {
    soroban_sdk::contractimport!(file = "../bytecodes/soroban_token_contract_phoenix.wasm");
    pub type PhoenixTokenClient<'a> = Client<'a>;
}
use phoenix_token::PhoenixTokenClient;

// relevant comet clients
pub mod comet_factory {
    soroban_sdk::contractimport!(file = "../bytecodes/comet_factory.wasm");
    pub type CometFactoryClient<'a> = Client<'a>;
}
use comet_factory::CometFactoryClient;
pub mod comet_pool {
    soroban_sdk::contractimport!(file = "../bytecodes/comet_pool.wasm");
    pub type CometPoolClient<'a> = Client<'a>;
}
use comet_pool::CometPoolClient;

// Hoops Adapter Clients
pub mod soroswap_adapter {
    soroban_sdk::contractimport!(file = "../bytecodes/soroswap_adapter.wasm");
    pub type SoroswapAdapterClient<'a> = Client<'a>;
}
use soroswap_adapter::SoroswapAdapterClient;
pub mod aqua_adapter {
    soroban_sdk::contractimport!(file = "../bytecodes/aqua_adapter.wasm");
    pub type AquaAdapterClient<'a> = Client<'a>;
}
use aqua_adapter::AquaAdapterClient;
pub mod phoenix_adapter {
    soroban_sdk::contractimport!(file = "../bytecodes/phoenix_adapter.wasm");
    pub type PhoenixAdapterClient<'a> = Client<'a>;
}
use phoenix_adapter::PhoenixAdapterClient;
pub mod comet_adapter {
    soroban_sdk::contractimport!(file = "../bytecodes/comet_adapter.wasm");
    pub type CometAdapterClient<'a> = Client<'a>;
}
use comet_adapter::CometAdapterClient;

// Hoops Router Client
pub mod hoops_router {
    soroban_sdk::contractimport!(file = "../bytecodes/hoops_router.wasm");
    pub type HoopsRouterClient<'a> = Client<'a>;
}
use hoops_router::HoopsRouterClient;

pub struct TokenContracts {
    /*
    pub client_a: &'a token::Client<'static>,
    pub client_b: &'a token::Client<'static>,
    pub client_c: &'a token::Client<'static>,
     */
    pub client_a: Address,
    pub client_b: Address,
    pub client_c: Address,
    }

pub struct AmmInfrastructure {
    pub name: String, // e.g., "Soroswap", "Aqua"
    pub factory_id: Address,
    pub router_id: Option<Address>, // Some AMMs might only have a factory or a combined entity
    pub pool_ids: Vec<Address>,
}

pub struct AdapterContracts {
    pub soroswap: SoroswapAdapterClient<'static>,
    pub aqua: AquaAdapterClient<'static>,
    pub phoenix: PhoenixAdapterClient<'static>,
    pub comet: CometAdapterClient<'static>,
}

pub struct HoopsTestEnvironment<'a> {
    pub env: Env,
    pub admin: Address,
    pub user: Address,
    pub tokens: TokenContracts,
    pub soroswap: AmmInfrastructure,
    pub aqua: AmmInfrastructure,
    pub phoenix: AmmInfrastructure,
    pub comet: AmmInfrastructure,
    pub adapters: AdapterContracts,
    pub router: HoopsRouterClient<'a>,
}

// create stellar token.
pub fn create_stellar_token<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let wasm = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &wasm.address()),
        token::StellarAssetClient::new(e, &wasm.address()),
    )
}

pub fn deploy_phoenix_factory_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
) -> PhoenixFactoryClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));

    let phoenix_multihop_wasm_hash = env.deployer().upload_contract_wasm(phoenix_multihop::WASM);
    let phoenix_pool_wasm_hash = env.deployer().upload_contract_wasm(phoenix_pool::WASM);
    let phoenix_pool_stable_wasm_hash = env
        .deployer()
        .upload_contract_wasm(phoenix_pool_stable::WASM);
    let phoenix_stake_wasm_hash = env.deployer().upload_contract_wasm(phoenix_stake::WASM);
    let phoenix_token_wasm_hash = env.deployer().upload_contract_wasm(phoenix_token::WASM);

    let whitelisted_accounts: Vec<Address> = vec![env, admin.clone()];

    let phoenixfactory = PhoenixFactoryClient::new(
        env,
        &env.register(
            phoenix_factory::WASM, //Factory, // previously this was native now it uses the wasm.
            (
                &admin,
                &phoenix_multihop_wasm_hash,
                &phoenix_pool_wasm_hash,
                &phoenix_pool_stable_wasm_hash,
                &phoenix_stake_wasm_hash,
                &phoenix_token_wasm_hash,
                whitelisted_accounts,
                &10u32,
            ),
        ),
    );

    phoenixfactory
}

pub fn set_phoenix_amm_infra(
    env: &Env,
    admin: Address,
    user: Address,
    phoenix_factory: &PhoenixFactoryClient<'_>,
    token_a_client: &token::Client,
    token_b_client: &token::Client,
    token_c_client: &token::Client,
) -> AmmInfrastructure {
    let mut phoenix_pool_ids = Vec::new(&env);
    // --- Phoenix Setup --- (should be it's own function later.)
    let pho_ab_lp_init_info = generate_pho_lp_init_info(
        token_a_client.address.clone(),
        token_b_client.address.clone(),
        admin.clone(),
        admin.clone(),
        user.clone(),
    );
    
    let pho_bc_lp_init_info = generate_pho_lp_init_info(
        token_b_client.address.clone(),
        token_c_client.address.clone(),
        admin.clone(),
        admin.clone(),
        user.clone(),
    );
    std::println!("Phoenix LP Init Info AB: {:?}", pho_ab_lp_init_info);
    std::println!("Phoenix LP Init Info BC: {:?}", pho_bc_lp_init_info);

    let phoenix_constant_pool_ab_address = phoenix_factory.create_liquidity_pool(
        &admin,
        &pho_ab_lp_init_info,
        &String::from_str(&env, "AB Constant"),
        &String::from_str(&env, "TKNA/TKNB"),
        //&PhoenixPoolType::Constant,
        &phoenix_factory::PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );
    std::println!("Phoenix constant pool AB address: {:?}", phoenix_constant_pool_ab_address);
    let phoenix_pool_stable_bc_address = phoenix_factory.create_liquidity_pool(
        &admin,
        &pho_bc_lp_init_info,
        &String::from_str(&env, "BC Stable"),
        &String::from_str(&env, "TKNB/TKNC"),
        &phoenix_factory::PoolType::Stable,
        &Some(10),
        &100i64,
        &1_000,
    );
    std::println!("Phoenix stable pool BC address: {:?}", phoenix_pool_stable_bc_address);
    std::println!("Phoenix Pools created: {:?}, {:?}", phoenix_constant_pool_ab_address, phoenix_pool_stable_bc_address);
    phoenix_pool_ids.push_back(phoenix_constant_pool_ab_address.clone());
    phoenix_pool_ids.push_back(phoenix_pool_stable_bc_address.clone());
    let phoenix_amm = AmmInfrastructure {
        name: "Phoenix".into_val(env),
        factory_id: phoenix_factory.address.clone(),
        router_id: None,
        pool_ids: phoenix_pool_ids,
    };
    phoenix_amm
}

pub fn setup_soroswap_environment<'a>(
    env: &'a Env,
    admin: &Address,
    user: &Address,
    token_a: &Address,
    token_b: &Address,
    token_c: &Address,
    initial_mint_amount: i128,
) -> (AmmInfrastructure, SoroswapRouterClient<'a>, Address) {
    // --- Soroswap Setup ---
    // Pool AB: 7.01:1 ratio (7.01M A, 1M B)
    let soroswap_ab_a = 7_010_000_000_000i128;
    let soroswap_ab_b = 1_000_000_000_000i128;
    // Pool BC: 1.01:1 ratio (1.01M B, 1M C)
    let soroswap_bc_b = 1_010_000_000_000i128;
    let soroswap_bc_c = 1_000_000_000_000i128;

    let soroswap_factory_id = env.register(SOROSWAP_FACTORY_WASM, ());
    let soroswap_router_id = env.register(SOROSWAP_ROUTER_WASM, ());
    let soroswap_pair_wasm_hash = env.deployer().upload_contract_wasm(SOROSWAP_PAIR_WASM);

    let soroswap_factory = SoroswapFactoryClient::new(env, &soroswap_factory_id);
    let soroswap_router = SoroswapRouterClient::new(env, &soroswap_router_id);

    soroswap_factory.initialize(admin, &soroswap_pair_wasm_hash);
    soroswap_router.initialize(&soroswap_factory_id);

    let mut soroswap_pool_ids = Vec::new(env);

    // Pool 1: token_a/token_b
    soroswap_factory.create_pair(token_a, token_b);
    let pair_ab_address = soroswap_factory.get_pair(token_a, token_b);
    soroswap_pool_ids.push_back(pair_ab_address.clone());
    let token_a_client = TokenClient::new(env, token_a);
    let token_b_client = TokenClient::new(env, token_b);
    token_a_client.approve(admin, &soroswap_router.address, &soroswap_ab_a, &200);
    token_b_client.approve(admin, &soroswap_router.address, &soroswap_ab_b, &200);
    soroswap_router.add_liquidity(
        token_a,
        token_b,
        &soroswap_ab_a,
        &soroswap_ab_b,
        &0,
        &0,
        admin,
        &(env.ledger().timestamp() + 100),
    );

    // Pool 2: token_b/token_c
    soroswap_factory.create_pair(token_b, token_c);
    let pair_bc_address = soroswap_factory.get_pair(token_b, token_c);
    soroswap_pool_ids.push_back(pair_bc_address.clone());
    let token_c_client = TokenClient::new(env, token_c);
    token_b_client.approve(admin, &soroswap_router.address, &soroswap_bc_b, &200);
    token_c_client.approve(admin, &soroswap_router.address, &soroswap_bc_c, &200);
    soroswap_router.add_liquidity(
        token_b,
        token_c,
        &soroswap_bc_b,
        &soroswap_bc_c,
        &0,
        &0,
        admin,
        &(env.ledger().timestamp() + 100),
    );
    std::println!("[Soroswap] Minting {} of token A and {} of token B for Pool AB", soroswap_ab_a, soroswap_ab_b);
    mint_and_approve(&env, &admin, &token_a, soroswap_ab_a, &pair_ab_address);
    mint_and_approve(&env, &admin, &token_b, soroswap_ab_b, &pair_ab_address);
    std::println!("[Soroswap] Depositing to Pool AB: {} A, {} B", soroswap_ab_a, soroswap_ab_b);
    provide_liquidity_soroswap(&env, &pair_ab_address, &admin, &token_a, &token_b, soroswap_ab_a, soroswap_ab_b);
    std::println!("[Soroswap] Minting {} of token B and {} of token C for Pool BC", soroswap_bc_b, soroswap_bc_c);
    mint_and_approve(&env, &admin, &token_b, soroswap_bc_b, &pair_bc_address);
    mint_and_approve(&env, &admin, &token_c, soroswap_bc_c, &pair_bc_address);
    std::println!("[Soroswap] Depositing to Pool BC: {} B, {} C", soroswap_bc_b, soroswap_bc_c);
    provide_liquidity_soroswap(&env, &pair_bc_address, &admin, &token_b, &token_c, soroswap_bc_b, soroswap_bc_c);
    std::println!("soroswap environment setup complete. Pools: {:?}, {:?}, router: {:?}, factory: {:?}", pair_ab_address, pair_bc_address, soroswap_router_id, soroswap_factory_id);
    let soroswap_amm = AmmInfrastructure {
        name: "Soroswap".into_val(env),
        factory_id: soroswap_factory_id,
        router_id: Some(soroswap_router_id.clone()),
        pool_ids: soroswap_pool_ids,
    };
    (soroswap_amm, soroswap_router, soroswap_router_id)
}

pub fn setup_aqua_environment(
    env: &Env,
    admin: &Address,
    user: &Address,
    token_a: &Address,
    token_b: &Address,
    token_c: &Address,
) -> (AmmInfrastructure, Address) {
    // --- Aqua Setup ---
    // Pool AB: 6.999:1 ratio (6.999M A, 1M B)
    let aqua_ab_a = 6_999_000_000_000i128;
    let aqua_ab_b = 1_000_000_000_000i128;
    // Pool BC: 1:0.999 ratio (1M B, 0.999M C)
    let aqua_bc_b = 1_000_000_000_000i128;
    let aqua_bc_c = 999_000_000_000i128;

    let aqua_router_id = env.register(AQUA_ROUTER_WASM, ());
    let aqua_router = AquaRouterClient::new(env, &aqua_router_id);
    let pool_hash = env.deployer().upload_contract_wasm(AQUA_POOL_CONSTANT_WASM);
    let stableswap_pool_hash = env.deployer().upload_contract_wasm(AQUA_STABLE_POOL_WASM);
    let token_hash = env.deployer().upload_contract_wasm(TOKEN_WASM);
    let reward_token = TokenClient::new(env, &env.register_stellar_asset_contract_v2(admin.clone()).address());
    let reward_boost_token = TokenClient::new(env, &env.register_stellar_asset_contract_v2(admin.clone()).address());
    // Mint reward tokens to the user for pool creation payment
    let initial_mint_amount: i128 = 10_000_000 * 10_000_000;
    reward_token.mint(user, &initial_mint_amount);
    reward_boost_token.mint(user, &initial_mint_amount);
    aqua_router.init_admin(admin);
    aqua_router.set_privileged_addrs(
        admin,
        &admin.clone(),
        &admin.clone(),
        &admin.clone(),
        &Vec::from_array(env, [admin.clone().clone()]),
    );
    aqua_router.set_pool_hash(admin, &pool_hash);
    aqua_router.set_stableswap_pool_hash(admin, &stableswap_pool_hash);
    aqua_router.set_token_hash(admin, &token_hash);
    aqua_router.set_reward_token(admin, &reward_token.address);
    aqua_router.configure_init_pool_payment(
        admin,
        &reward_token.address,
        &1_0000000,
        &1_0000000,
        &aqua_router.address,
    );
    
    let reward_boost_feed = create_reward_boost_feed_contract(env, &admin, &admin, &admin);
    
    aqua_router.set_reward_boost_config(admin, &reward_boost_token.address, &reward_boost_feed.address);

    let plane_contract = create_plane_contract(env);
    aqua_router.set_pools_plane(admin, &plane_contract.address);
    let mut aqua_pool_ids = Vec::new(&env);
    let initial_mint_amount: i128 = 10_000_000 * 10_000_000;
    let token_a_client = TokenClient::new(&env, token_a);
    let token_b_client = TokenClient::new(&env, token_b);
    let token_c_client = TokenClient::new(&env, token_c);
    // Approve the router to spend user's reward tokens for pool creation
    reward_token.approve(user, &aqua_router.address, &initial_mint_amount, &200);
    reward_boost_token.approve(user, &aqua_router.address, &initial_mint_amount, &200);
    // Approve the router to spend user's tokens for pool creation
    token_a_client.approve(user, &aqua_router.address, &(initial_mint_amount / 2), &200);
    token_b_client.approve(user, &aqua_router.address, &(initial_mint_amount / 2), &200);
    token_c_client.approve(user, &aqua_router.address, &(initial_mint_amount / 2), &200);
    let aqua_tokens = Vec::from_array(env, [token_a.clone(), token_b.clone()]);
    let (aqua_pool_ab_hash, aqua_pool_ab_address) = aqua_router.init_standard_pool(user, &aqua_tokens, &30);
    aqua_pool_ids.push_back(aqua_pool_ab_address.clone());
    let aqua_stable_tokens = Vec::from_array(env, [token_b.clone(), token_c.clone()]);
    let (_aqua_stable_bc_hash, aqua_stable_bc_address) = aqua_router.init_stableswap_pool(user, &aqua_stable_tokens, &10);
    aqua_pool_ids.push_back(aqua_stable_bc_address.clone());
    // Approve the pool contracts for deposit (after creation)
    token_a_client.approve(user, &aqua_pool_ab_address, &(initial_mint_amount / 2), &200);
    token_b_client.approve(user, &aqua_pool_ab_address, &(initial_mint_amount / 2), &200);
    token_b_client.approve(user, &aqua_stable_bc_address, &(initial_mint_amount / 2), &200);
    token_c_client.approve(user, &aqua_stable_bc_address, &(initial_mint_amount / 2), &200);
    let deposit_amounts_ab = vec![env, initial_mint_amount as u128 / 10, initial_mint_amount as u128 / 10];
    let _ = aqua_router.deposit(user, &aqua_tokens, &aqua_pool_ab_hash, &deposit_amounts_ab, &0);
    let deposit_amounts_bc = vec![env, initial_mint_amount as u128 / 10, initial_mint_amount as u128 / 10];
    let _ = aqua_router.deposit(user, &aqua_stable_tokens, &_aqua_stable_bc_hash, &deposit_amounts_bc, &0);
    std::println!("[Aqua] Minting {} of token A and {} of token B for Pool AB", aqua_ab_a, aqua_ab_b);
    mint_and_approve(&env, &user, &token_a, aqua_ab_a, &aqua_pool_ab_address);
    mint_and_approve(&env, &user, &token_b, aqua_ab_b, &aqua_pool_ab_address);
    std::println!("[Aqua] Depositing to Pool AB: {} A, {} B", aqua_ab_a, aqua_ab_b);
    provide_liquidity_aqua(&env, &aqua_pool_ab_address, &user, &token_a, &token_b, aqua_ab_a, aqua_ab_b);
    std::println!("[Aqua] Minting {} of token B and {} of token C for Pool BC", aqua_bc_b, aqua_bc_c);
    mint_and_approve(&env, &user, &token_b, aqua_bc_b, &aqua_stable_bc_address);
    mint_and_approve(&env, &user, &token_c, aqua_bc_c, &aqua_stable_bc_address);
    std::println!("[Aqua] Depositing to Pool BC: {} B, {} C", aqua_bc_b, aqua_bc_c);
    provide_liquidity_aqua(&env, &aqua_stable_bc_address, &user, &token_b, &token_c, aqua_bc_b, aqua_bc_c);
    std::println!("Aqua pools created: {:?}, {:?}", aqua_pool_ab_address, aqua_stable_bc_address);
    (AmmInfrastructure {
        name: "Aqua".into_val(env),
        factory_id: aqua_router_id.clone(),
        router_id: Some(aqua_router_id.clone()),
        pool_ids: aqua_pool_ids,
    }, aqua_router_id)
}

pub fn setup_comet_environment(
    env: &Env,
    admin: &Address,
    user: &Address,
    token_a: &Address,
    token_b: &Address,
    token_c: &Address,
    initial_mint_amount: i128,
) -> AmmInfrastructure {
    // --- Comet Setup ---
    // Pool AB: 7:1 price, 80/20 weights
    let comet_ab_a = 2_800_000_000_000i128; // 2.8M A (7 decimals)
    let comet_ab_b = 100_000_000_000i128;   // 0.1M B (7 decimals)
    // Pool BC: 1:1 price, 50/50 weights
    let comet_bc_b = 1_000_000_000_000i128;
    let comet_bc_c = 1_000_000_000_000i128;

    std::println!("[Comet] Registering factory contract");
    let comet_factory_id = env.register(COMET_FACTORY_WASM, ());
    let comet_factory = CometFactoryClient::new(env, &comet_factory_id);
    std::println!("[Comet] Uploading pool WASM");
    let comet_pool_wasm_hash = env.deployer().upload_contract_wasm(COMET_POOL_WASM);
    std::println!("[Comet] Initializing factory");
    comet_factory.init(&comet_pool_wasm_hash);
    let mut comet_pool_ids = Vec::new(env);
    // --- Robust Comet Pool Setup ---
    // Pool 1: token_a/token_b, 80/20, 7:1 price
    let tokens_ab = vec![env, token_a.clone(), token_b.clone()];
    let weights_ab = vec![env, 8_000_000i128, 2_000_000i128];
    let balances_ab = vec![env, comet_ab_a, comet_ab_b];
    let salt_ab = BytesN::from_array(env, &[0; 32]);
    let swap_fee = 3_000i128; // 0.03% fee, adjust as needed
    std::println!("[Comet] Creating pool: token_a/token_b (80/20, 7:1 price)");
    let pool_ab = comet_factory.new_c_pool(
        &salt_ab,
        admin,
        &tokens_ab,
        &weights_ab,
        &balances_ab,
        &swap_fee,
    );
    std::println!("[Comet] Pool AB address: {:?}", pool_ab);
    comet_pool_ids.push_back(pool_ab.clone());
    // Pool 2: token_b/token_c, 50/50, 1:1 price
    let tokens_bc = vec![env, token_b.clone(), token_c.clone()];
    let weights_bc = vec![env, 5_000_000i128, 5_000_000i128];
    let balances_bc = vec![env, comet_bc_b, comet_bc_c];
    let salt_bc = BytesN::from_array(env, &[1; 32]);
    std::println!("[Comet] Creating pool: token_b/token_c (50/50, 1:1 price)");
    let pool_bc = comet_factory.new_c_pool(
        &salt_bc,
        admin,
        &tokens_bc,
        &weights_bc,
        &balances_bc,
        &swap_fee,
    );
    std::println!("[Comet] Pool BC address: {:?}", pool_bc);
    comet_pool_ids.push_back(pool_bc.clone());
    std::println!("[Comet] Minting {} of token A and {} of token B for Pool AB", comet_ab_a, comet_ab_b);
    mint_and_approve(&env, &user, &token_a, comet_ab_a, &pool_ab);
    mint_and_approve(&env, &user, &token_b, comet_ab_b, &pool_ab);
    std::println!("[Comet] Depositing to Pool AB: {} A, {} B", comet_ab_a, comet_ab_b);
    provide_liquidity_comet(&env, &pool_ab, &user, &token_a, &token_b, comet_ab_a, comet_ab_b);
    std::println!("[Comet] Minting {} of token B and {} of token C for Pool BC", comet_bc_b, comet_bc_c);
    mint_and_approve(&env, &user, &token_b, comet_bc_b, &pool_bc);
    mint_and_approve(&env, &user, &token_c, comet_bc_c, &pool_bc);
    std::println!("[Comet] Depositing to Pool BC: {} B, {} C", comet_bc_b, comet_bc_c);
    provide_liquidity_comet(&env, &pool_bc, &user, &token_b, &token_c, comet_bc_b, comet_bc_c);
    std::println!("[Comet] All pools created: {:?}", comet_pool_ids);
    AmmInfrastructure {
        name: "Comet".into_val(env),
        factory_id: comet_factory_id,
        router_id: None,
        pool_ids: comet_pool_ids,
    }
}

pub fn setup_test_accounts(env: &Env) -> (Address, Address) {
    let admin = Address::generate(env);
    let user = Address::generate(env);

    (admin, user)
}

// Utility function to print pool reserves for a given AMM
fn print_amm_pools_and_reserves(env: &Env, amm: &AmmInfrastructure, label: &str, user: &Address) {
    std::println!("[REPORT] {} Pools:", label);
    for pool_addr in amm.pool_ids.iter() {
        std::println!("[REPORT]   Pool: {:?}", pool_addr);
        if label == "Soroswap" {
            let pool = SoroswapPairClient::new(env, &pool_addr);
            let token0 = pool.token_0();
            let token1 = pool.token_1();
            let (reserve_0, reserve_1) = pool.get_reserves();
            let lp_token = pool.address;
            let user_lp_balance = TokenClient::new(env, &lp_token).balance(user);
            std::println!("[REPORT - Soroswap]     Tokens: {:?}, {:?}", token0, token1);
            std::println!("[REPORT - Soroswap]     Reserves: {:?}, {:?}", reserve_0, reserve_1);
            std::println!("[REPORT - Soroswap]     LP Token: {:?} User LP Balance: {:?}", lp_token, user_lp_balance);
        } else if label == "Aqua" {
            let pool = AquaPoolClient::new(env, &pool_addr);
            let lp_token_id = pool.share_id();
            let share_token = ShareTokenClient::new(env, &lp_token_id);
            let tokens = pool.get_tokens();
            let reserves = pool.get_reserves();
            let user_lp_balance = share_token.balance(&user);
            std::println!("[REPORT - Aqua]     Tokens: {:?}", tokens);
            std::println!("[REPORT - Aqua]     Reserves: {:?}", reserves);
            std::println!("[REPORT - Aqua]     Pool: {:?}  LP Token: {:?} User LP Balance: {:?}", pool.address, lp_token_id, user_lp_balance);
        } else if label == "Phoenix" {
            let pool = PhoenixPoolClient::new(env, &pool_addr);
            let info = pool.query_pool_info();
            let lp_token = info.asset_lp_share.address.clone();
            let user_lp_balance = TokenClient::new(env, &lp_token).balance(user);
            std::println!("[REPORT - Phoenix]     Pool Info: {:?}", info);
            std::println!("[REPORT - Phoenix]     Asset A: {:?} amount: {:?}", info.asset_a.address, info.asset_a.amount);
            std::println!("[REPORT - Phoenix]     Asset B: {:?} amount: {:?}", info.asset_b.address, info.asset_b.amount);
            std::println!("[REPORT - Phoenix]     LP Share: {:?} amount: {:?}", info.asset_lp_share.address, info.asset_lp_share.amount);
            std::println!("[REPORT - Phoenix]     User LP Balance: {:?}", user_lp_balance);
        } else if label == "Comet" {
            let pool = CometPoolClient::new(env, &pool_addr);
            let tokens = pool.get_tokens();
            std::println!("[REPORT - Comet]     Tokens: {:?}", tokens);
            for token in tokens.iter() {
                let bal = pool.get_balance(&token);
                std::println!("[REPORT - Comet]       Balance for token {:?}: {:?}", token, bal);
            }
            let lp_token = pool.address;
            let user_lp_balance = TokenClient::new(env, &lp_token).balance(user);
            std::println!("[REPORT - Comet]     LP Token: {:?} User LP Balance: {:?}", lp_token, user_lp_balance);
        }
    }
}

fn mint_and_approve(env: &Env, user: &Address, token: &Address, amount: i128, spender: &Address) {
    let token_client = TokenClient::new(env, token);
    token_client.mint(user, &amount);
    token_client.approve(user, spender, &amount, &200);
}

fn provide_liquidity_soroswap(env: &Env, pair: &Address, user: &Address, token_a: &Address, token_b: &Address, amount_a: i128, amount_b: i128) {
    let pair_client = SoroswapPairClient::new(env, pair);
    let token_a_client = TokenClient::new(env, token_a);
    let token_b_client = TokenClient::new(env, token_b);
    token_a_client.transfer(user, pair, &amount_a);
    token_b_client.transfer(user, pair, &amount_b);
}

fn provide_liquidity_aqua(env: &Env, pool: &Address, user: &Address, token_a: &Address, token_b: &Address, amount_a: i128, amount_b: i128) {
    let pool_client = AquaPoolClient::new(env, pool);
    let token_a_client = TokenClient::new(env, token_a);
    let token_b_client = TokenClient::new(env, token_b);
    token_a_client.transfer(user, pool, &amount_a);
    token_b_client.transfer(user, pool, &amount_b);
    pool_client.deposit(user, &vec![env, amount_a as u128, amount_b as u128], &0u128);
}

fn provide_liquidity_phoenix(env: &Env, pool: &Address, user: &Address, token_a: &Address, token_b: &Address, amount_a: i128, amount_b: i128) {
    let pool_client = PhoenixPoolClient::new(env, pool);
    let token_a_client = TokenClient::new(env, token_a);
    let token_b_client = TokenClient::new(env, token_b);
    token_a_client.transfer(user, pool, &amount_a);
    token_b_client.transfer(user, pool, &amount_b);
    // Always pass both amount and min_amount for each token
    pool_client.provide_liquidity(
        user,
        &Some(amount_a),
        &Some(amount_b),
        &Some(amount_a),
        &Some(amount_b),
        &None,
        &None::<u64>,
        &false,
    );
}

fn provide_liquidity_phoenix_stable(env: &Env, pool: &Address, user: &Address, token_a: &Address, token_b: &Address, amount_a: i128, amount_b: i128) {
    let pool_client = PhoenixPoolStableClient::new(env, pool);
    let token_a_client = TokenClient::new(env, token_a);
    let token_b_client = TokenClient::new(env, token_b);
    token_a_client.transfer(user, pool, &amount_a);
    token_b_client.transfer(user, pool, &amount_b);
    pool_client.provide_liquidity(user, &amount_a, &amount_b, &None, &None::<u64>, &None::<u128>, &false);
}

fn provide_liquidity_comet(env: &Env, pool: &Address, user: &Address, token_a: &Address, token_b: &Address, amount_a: i128, amount_b: i128) {
    let pool_client = CometPoolClient::new(env, pool);
    let token_a_client = TokenClient::new(env, token_a);
    let token_b_client = TokenClient::new(env, token_b);
    // Removed direct transfers to pool, unlike soroswap the pool pulls them automatically.
    token_a_client.approve(user, pool, &amount_a, &200);
    token_b_client.approve(user, pool, &amount_b, &200);
    // Use a positive pool_amount_out (e.g., 1_000_000)
    let pool_amount_out = 1_000_000i128;
    pool_client.join_pool(&pool_amount_out, &vec![env, amount_a, amount_b], user);
}

fn get_reserve_soroswap(pair: &Address, token: &Address) -> i128 {
    let env = pair.env();
    let pair_client = SoroswapPairClient::new(&env, pair);
    let (r0, r1) = pair_client.get_reserves();
    if pair_client.token_0() == *token { r0 } else { r1 }
}

fn get_lp_balance_soroswap(pair: &Address, user: &Address) -> i128 {
    let env = pair.env();
    let pair_client = SoroswapPairClient::new(&env, pair);
    pair_client.balance(user)
}

fn get_reserve_aqua(pool: &Address, token: &Address) -> i128 {
    let env = pool.env();
    let pool_client = AquaPoolClient::new(&env, pool);
    let tokens = pool_client.get_tokens();
    let reserves = pool_client.get_reserves();
    if tokens.get(0).unwrap() == *token { reserves.get(0).unwrap() as i128 } else { reserves.get(1).unwrap() as i128 }
}

fn get_lp_balance_aqua(pool: &Address, user: &Address) -> i128 {
    let env = pool.env();
    let pool_client = AquaPoolClient::new(&env, pool);
    let lp_token_id = pool_client.share_id();
    let share_token = ShareTokenClient::new(env, &lp_token_id);
    share_token.balance(user)
}

fn get_reserve_phoenix(pool: &Address, token: &Address) -> i128 {
    let env = pool.env();
    let pool_client = PhoenixPoolClient::new(&env, pool);
    let info = pool_client.query_pool_info();
    if info.asset_a.address == *token { info.asset_a.amount } else { info.asset_b.amount }
}

fn get_lp_balance_phoenix(pool: &Address, user: &Address) -> i128 {
    let env = pool.env();
    let pool_client = PhoenixPoolClient::new(&env, pool);
    let info = pool_client.query_pool_info();
    let lp_token = info.asset_lp_share.address;
    TokenClient::new(&env, &lp_token).balance(user)
}

fn get_balance_comet(pool: &Address, token: &Address) -> i128 {
    let env = pool.env();
    let pool_client = CometPoolClient::new(&env, pool);
    pool_client.get_balance(token)
}

fn get_share_balance_comet(pool: &Address, user: &Address) -> i128 {
    let env = pool.env();
    let pool_client = CometPoolClient::new(&env, pool);
    pool_client.balance(user)
}

impl<'a> HoopsTestEnvironment<'a> {
    pub fn setup() -> Self {
        let env = Env::default();
        std::println!("[SETUP] Starting HoopsTestEnvironment setup");
        env.mock_all_auths();
        env.cost_estimate().budget().reset_unlimited(); // Corrected budget reset

        // Deploy Tokens
        std::println!("[SETUP] Creating test accounts");
        let (admin, user) = setup_test_accounts(&env);

        std::println!("[SETUP] Deploying tokens");
        let (token_a_client, token_a_admin) = create_stellar_token(&env, &admin);
        std::println!("[LOG] Token A deployed at: {:?}", token_a_client.address);
        let (token_b_client, token_b_admin) = create_stellar_token(&env, &admin);
        std::println!("[LOG] Token B deployed at: {:?}", token_b_client.address);
        let (token_c_client, token_c_admin) = create_stellar_token(&env, &admin);
        std::println!("[LOG] Token C deployed at: {:?}", token_c_client.address);

        let initial_mint_amount: i128 = 10_000_000 * 10_000_000; // 10M tokens with 7 decimals

        std::println!("[SETUP] Minting tokens to admin and user");
        token_a_admin.mint(&admin, &initial_mint_amount); std::println!("[LOG] Minted {} TokenA to {:?}", initial_mint_amount, admin);
        token_b_admin.mint(&admin, &initial_mint_amount); std::println!("[LOG] Minted {} TokenB to {:?}", initial_mint_amount, admin);
        token_c_admin.mint(&admin, &initial_mint_amount); std::println!("[LOG] Minted {} TokenC to {:?}", initial_mint_amount, admin);
        token_a_admin.mint(&user, &initial_mint_amount); std::println!("[LOG] Minted {} TokenA to {:?}", initial_mint_amount, user);
        token_b_admin.mint(&user, &initial_mint_amount); std::println!("[LOG] Minted {} TokenB to {:?}", initial_mint_amount, user);
        token_c_admin.mint(&user, &initial_mint_amount); std::println!("[LOG] Minted {} TokenC to {:?}", initial_mint_amount, user);

        std::println!("[SETUP] Setting up Soroswap environment");
        let (soroswap_amm, soroswap_router, soroswap_router_id) = setup_soroswap_environment(
            &env, &admin, &user, &token_a_client.address, &token_b_client.address, &token_c_client.address, initial_mint_amount
        );
        std::println!("[LOG] Soroswap router deployed at: {:?}", soroswap_router_id);
        std::println!("[LOG] Soroswap factory deployed at: {:?}", soroswap_amm.factory_id);
        for (i, pool) in soroswap_amm.pool_ids.iter().enumerate() {
            std::println!("[LOG] Soroswap pool {} deployed at: {:?}", i, pool);
        }
        std::println!("[SETUP] Soroswap environment ready");

        std::println!("[SETUP] Setting up Aqua environment");
        let (aqua_amm, aqua_router_id) = setup_aqua_environment(
            &env, &admin, &user, &token_a_client.address, &token_b_client.address, &token_c_client.address
        );
        std::println!("[LOG] Aqua router deployed at: {:?}", aqua_router_id);
        std::println!("[LOG] Aqua factory (router) deployed at: {:?}", aqua_amm.factory_id);
        for (i, pool) in aqua_amm.pool_ids.iter().enumerate() {
            std::println!("[LOG] Aqua pool {} deployed at: {:?}", i, pool);
        }
        std::println!("[SETUP] Aqua environment ready");

        std::println!("[SETUP] Deploying Phoenix factory");
        let phoenix_factory = deploy_phoenix_factory_contract(&env, Some(admin.clone()));
        let phoenix_factory_id = phoenix_factory.address.clone();
        std::println!("[LOG] Phoenix factory deployed at: {:?}", phoenix_factory_id);
        std::println!("[SETUP] Setting up Phoenix pools");
        let phoenix_amm = set_phoenix_amm_infra(
            &env,
            admin.clone(),
            user.clone(),
            &phoenix_factory,
            &token_a_client,
            &token_b_client,
            &token_c_client,
        );
        for (i, pool) in phoenix_amm.pool_ids.iter().enumerate() {
            std::println!("[LOG] Phoenix pool {} deployed at: {:?}", i, pool);
        }
        let phoenix_ab_a = 7_000_000_000_000i128;
        let phoenix_ab_b = 1_000_000_000_000i128;
        let phoenix_bc_b = 999_000_000_000i128;
        let phoenix_bc_c = 1_000_000_000_000i128;
        std::println!("[Phoenix] Minting {} of token A and {} of token B for Pool AB", phoenix_ab_a, phoenix_ab_b);
        mint_and_approve(&env, &admin, &token_a_client.address, phoenix_ab_a, &phoenix_amm.pool_ids.get(0).unwrap());
        mint_and_approve(&env, &admin, &token_b_client.address, phoenix_ab_b, &phoenix_amm.pool_ids.get(0).unwrap());
        std::println!("[Phoenix] Depositing to Pool AB: {} A, {} B", phoenix_ab_a, phoenix_ab_b);
        provide_liquidity_phoenix(&env, &phoenix_amm.pool_ids.get(0).unwrap(), &admin, &token_a_client.address, &token_b_client.address, phoenix_ab_a, phoenix_ab_b);
        std::println!("[Phoenix] Minting {} of token B and {} of token C for Pool BC", phoenix_bc_b, phoenix_bc_c);
        mint_and_approve(&env, &admin, &token_b_client.address, phoenix_bc_b, &phoenix_amm.pool_ids.get(1).unwrap());
        mint_and_approve(&env, &admin, &token_c_client.address, phoenix_bc_c, &phoenix_amm.pool_ids.get(1).unwrap());
        std::println!("[Phoenix] Depositing to Pool BC: {} B, {} C", phoenix_bc_b, phoenix_bc_c);
        provide_liquidity_phoenix_stable(&env, &phoenix_amm.pool_ids.get(1).unwrap(), &admin, &token_b_client.address, &token_c_client.address, phoenix_bc_b, phoenix_bc_c);
        std::println!("[SETUP] Phoenix environment ready");

        // --- Comet Setup ---
        std::println!("[SETUP] Setting up Comet environment");
        let comet_amm = setup_comet_environment(
            &env,
            &admin,
            &user,
            &token_a_client.address,
            &token_b_client.address,
            &token_c_client.address,
            initial_mint_amount,
        );
        std::println!("[LOG] Comet factory deployed at: {:?}", comet_amm.factory_id);
        for (i, pool) in comet_amm.pool_ids.iter().enumerate() {
            std::println!("[LOG] Comet pool {} deployed at: {:?}", i, pool);
        }
        std::println!("[SETUP] Comet environment ready");

        // --- Deploy Adapters ---
        std::println!("[SETUP] Deploying Soroswap adapter");
        let soroswap_adapter_id = env.register(SOROSWAP_ADAPTER_WASM, ());
        let soroswap_adapter = SoroswapAdapterClient::new(&env, &soroswap_adapter_id);
        soroswap_adapter.initialize(&3, &soroswap_router_id);
        std::println!("[LOG] Soroswap adapter deployed at: {:?}", soroswap_adapter_id);
        std::println!("[SETUP] Soroswap adapter initialized");

        std::println!("[SETUP] Deploying Aqua adapter");
        let aqua_adapter_id = env.register(AQUA_ADAPTER_WASM, ());
        let aqua_adapter = AquaAdapterClient::new(&env, &aqua_adapter_id);
        aqua_adapter.initialize(&0, &aqua_router_id);
        std::println!("[LOG] Aqua adapter deployed at: {:?}", aqua_adapter_id);
        std::println!("[SETUP] Aqua adapter initialized");

        std::println!("[SETUP] Deploying Phoenix adapter");
        let phoenix_adapter_id = env.register(PHOENIX_ADAPTER_WASM, ());
        let phoenix_adapter = PhoenixAdapterClient::new(&env, &phoenix_adapter_id);
        if let Some(first_phoenix_pool) = phoenix_amm.pool_ids.get(0) {
            phoenix_adapter.initialize(&2, &first_phoenix_pool);
        } else {
            phoenix_adapter.initialize(&2, &phoenix_factory_id);
        }
        std::println!("[LOG] Phoenix adapter deployed at: {:?}", phoenix_adapter_id);
        std::println!("[SETUP] Phoenix adapter initialized");

        std::println!("[SETUP] Deploying Comet adapter");
        let comet_adapter_id = env.register(COMET_ADAPTER_WASM, ());
        let comet_adapter = CometAdapterClient::new(&env, &comet_adapter_id);
        if let Some(first_comet_pool) = comet_amm.pool_ids.get(0) {
            comet_adapter.initialize(&1, &first_comet_pool);
        } else {
            comet_adapter.initialize(&1, &comet_amm.factory_id);
        }
        std::println!("[LOG] Comet adapter deployed at: {:?}", comet_adapter_id);
        std::println!("[SETUP] Comet adapter initialized");

        // Register all Comet pools with the adapter
        for pool_addr in comet_amm.pool_ids.iter() {
            let pool_client = CometPoolClient::new(&env, &pool_addr);
            let tokens = pool_client.get_tokens();
            comet_adapter.set_pool_for_tokens(&tokens, &pool_addr);
            std::println!("[SETUP] Registered Comet pool {:?} for tokens {:?}", pool_addr, tokens);
        }

        let adapters = AdapterContracts {
            soroswap: soroswap_adapter,
            aqua: aqua_adapter,
            phoenix: phoenix_adapter,
            comet: comet_adapter,
        };

        // --- Deploy Hoops Router ---
        std::println!("[SETUP] Deploying Hoops router");
        let hoops_router_id = env.register(HOOPS_ROUTER_WASM, ());
        let hoops_router = HoopsRouterClient::new(&env, &hoops_router_id);
        hoops_router.initialize(&admin);
        std::println!("[LOG] Hoops router deployed at: {:?}", hoops_router_id);
        std::println!("[SETUP] Hoops router initialized");

        std::println!("[SETUP] HoopsTestEnvironment setup complete");
        // --- Print summary report ---
        print_amm_pools_and_reserves(&env, &soroswap_amm, "Soroswap", &user);
        print_amm_pools_and_reserves(&env, &aqua_amm, "Aqua", &user);
        print_amm_pools_and_reserves(&env, &phoenix_amm, "Phoenix", &user);
        print_amm_pools_and_reserves(&env, &comet_amm, "Comet", &user);
        Self {
            env,
            admin,
            user,
            tokens: TokenContracts {
                client_a: token_a_client.address.clone(),
                client_b: token_b_client.address.clone(),
                client_c: token_c_client.address.clone(),
            },
            soroswap: soroswap_amm,
            aqua: aqua_amm,
            phoenix: phoenix_amm,
            comet: comet_amm,
            adapters,
            router: hoops_router,
        }
    }
}

/*
#[test]
fn test_environment_setup_placeholders() {
    let _test_env = HoopsTestEnvironment::setup();
    
}*/

#[test]
fn test_setup_verification() {
    let test_env = HoopsTestEnvironment::setup();
    // Soroswap
    let pair_ab = &test_env.soroswap.pool_ids.get(0).unwrap();
    let pair_bc = &test_env.soroswap.pool_ids.get(1).unwrap();
    let user = &test_env.user;
    let token_a = &test_env.tokens.client_a;
    let token_b = &test_env.tokens.client_b;
    let token_c = &test_env.tokens.client_c;
    std::println!("[TEST] Verifying Soroswap Pool AB reserves and LP balance");
    assert!(get_reserve_soroswap(pair_ab, token_a) > 0);
    assert!(get_reserve_soroswap(pair_ab, token_b) > 0);
    assert!(get_lp_balance_soroswap(pair_ab, user) > 0);
    assert!(get_reserve_soroswap(pair_bc, token_b) > 0);
    assert!(get_reserve_soroswap(pair_bc, token_c) > 0);
    assert!(get_lp_balance_soroswap(pair_bc, user) > 0);

    // Aqua
    let pool_ab = &test_env.aqua.pool_ids.get(0).unwrap();
    let pool_bc = &test_env.aqua.pool_ids.get(1).unwrap();
    std::println!("[TEST] Verifying Aqua Pool AB reserves and LP balance");
    assert!(get_reserve_aqua(pool_ab, token_a) > 0);
    assert!(get_reserve_aqua(pool_ab, token_b) > 0);
    assert!(get_lp_balance_aqua(pool_ab, user) > 0);
    assert!(get_reserve_aqua(pool_bc, token_b) > 0);
    assert!(get_reserve_aqua(pool_bc, token_c) > 0);
    assert!(get_lp_balance_aqua(pool_bc, user) > 0);
    // Phoenix
    let pho_ab = &test_env.phoenix.pool_ids.get(0).unwrap();
    let pho_bc = &test_env.phoenix.pool_ids.get(1).unwrap();
    std::println!("[TEST] Verifying Phoenix Pool AB reserves and LP balance");
    assert!(get_reserve_phoenix(pho_ab, token_a) > 0);
    assert!(get_reserve_phoenix(pho_ab, token_b) > 0);
    assert!(get_lp_balance_phoenix(pho_ab, user) > 0);
    assert!(get_reserve_phoenix(pho_bc, token_b) > 0);
    assert!(get_reserve_phoenix(pho_bc, token_c) > 0);
    assert!(get_lp_balance_phoenix(pho_bc, user) > 0);
    // Comet
    let comet_ab = &test_env.comet.pool_ids.get(0).unwrap();
    let comet_bc = &test_env.comet.pool_ids.get(1).unwrap();
    std::println!("[TEST] Verifying Comet Pool AB reserves and LP balance");
    assert!(get_balance_comet(comet_ab, token_a) > 0);
    assert!(get_balance_comet(comet_ab, token_b) > 0);
    assert!(get_share_balance_comet(comet_ab, user) > 0);
    assert!(get_balance_comet(comet_bc, token_b) > 0);
    assert!(get_balance_comet(comet_bc, token_c) > 0);
    assert!(get_share_balance_comet(comet_bc, user) > 0);
}

#[test]
fn test_all_adapters_all_functions() {
    use std::panic::AssertUnwindSafe;
    
    let test_env = HoopsTestEnvironment::setup();
    let mut failures = 0;
    // Soroswap
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::soroswap_adapter_tests::run_swap_exact_in(&test_env))) {
        std::println!("[FAIL][SOROSWAP][swap_exact_in]: {:?}", e); failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::soroswap_adapter_tests::run_swap_exact_out(&test_env))) {
        std::println!("[FAIL][SOROSWAP][swap_exact_out]: {:?}", e); failures += 1;
    }

    let lp = match std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::soroswap_adapter_tests::run_add_liquidity(&test_env))) {
        Ok(lp) => lp,
        Err(e) => {
            std::println!("[FAIL][SOROSWAP][add_liquidity]: {:?}", e); failures += 1;
            0 // Default value if the test fails
        }
    };
    // Only run remove_liquidity if add_liquidity succeeded (lp > 0)
    if lp > 0 {
        if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::soroswap_adapter_tests::run_remove_liquidity(&test_env, lp))) {
            std::println!("[FAIL][SOROSWAP][remove_liquidity]: {:?}", e); failures += 1;
        }
    } else {
        std::println!("[INFO][SOROSWAP] add liquidity failed, skipping remove");
    }
    // Comet
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::comet_adapter_tests::run_swap_exact_in(&test_env))) {
        std::println!("[FAIL][COMET][swap_exact_in]: {:?}", e); failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::comet_adapter_tests::run_swap_exact_out(&test_env))) {
        std::println!("[FAIL][COMET][swap_exact_out]: {:?}", e); failures += 1;
    }
    
    // Run add_liquidity and capture the LP token amount
    let lp = match std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::comet_adapter_tests::run_add_liquidity(&test_env))) {
        Ok(lp) => lp,
        Err(e) => {
            std::println!("[FAIL][COMET][add_liquidity]: {:?}", e); failures += 1;
            0 // Default value if the test fails
        }
    };

    // Only run remove_liquidity if add_liquidity succeeded (lp > 0)
    if lp > 0 {
        if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::comet_adapter_tests::run_remove_liquidity(&test_env, lp))) {
            std::println!("[FAIL][COMET][remove_liquidity]: {:?}", e); failures += 1;
        }
    } else {
        std::println!("[INFO][COMET] add liquidity failed, skipping remove");
    }
    // Aqua
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::aqua_adapter_tests::run_swap_exact_in(&test_env))) {
        std::println!("[FAIL][AQUA][swap_exact_in]: {:?}", e); failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::aqua_adapter_tests::run_swap_exact_out(&test_env))) {
        std::println!("[FAIL][AQUA][swap_exact_out]: {:?}", e); failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::aqua_adapter_tests::run_add_liquidity(&test_env))) {
        std::println!("[FAIL][AQUA][add_liquidity]: {:?}", e); failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::aqua_adapter_tests::run_remove_liquidity(&test_env))) {
        std::println!("[FAIL][AQUA][remove_liquidity]: {:?}", e); failures += 1;
    }
    // Phoenix
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::phoenix_adapter_tests::run_swap_exact_in(&test_env))) {
        std::println!("[FAIL][PHOENIX][swap_exact_in]: {:?}", e); failures += 1;
    }
    if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::phoenix_adapter_tests::run_swap_exact_out(&test_env))) {
        std::println!("[FAIL][PHOENIX][swap_exact_out]: {:?}", e); failures += 1;
    }
    // Run add_liquidity and capture the LP token amount
    let lp = match std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::phoenix_adapter_tests::run_add_liquidity(&test_env))) {
        Ok(lp) => lp,
        Err(e) => {
            std::println!("[FAIL][PHOENIX][add_liquidity]: {:?}", e); failures += 1;
            0 // Default value if the test fails
        }
    };
    // Only run remove_liquidity if add_liquidity succeeded (lp > 0)
    if lp > 0 {
        if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| crate::tests::phoenix_adapter_tests::run_remove_liquidity(&test_env, lp))) {
            std::println!("[FAIL][PHOENIX][remove_liquidity]: {:?}", e); failures += 1;
        }
    } else {
        std::println!("[INFO][PHOENIX] add liquidity failed, skipping remove");
    }
    if failures > 0 {
        panic!("{} adapter subtests failed. See log for details.", failures);
    }
}
