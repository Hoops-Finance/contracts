#![cfg(test)]
#![allow(unused_imports, unused_variables)]
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
use phoenix_pool::PhoenixPoolClient;

pub mod phoenix_pool_stable {
    soroban_sdk::contractimport!(file = "../bytecodes/phoenix_pool_stable.wasm");
    pub type PhoenixPoolStableClient<'a> = Client<'a>;
}
use phoenix_pool_stable::PhoenixPoolStableClient;

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
    let phoenix_constant_pool_ab_address = phoenix_factory.create_liquidity_pool(
        &admin,
        &pho_ab_lp_init_info,
        &String::from_str(&env, "phoenix_pool_ab"),
        &String::from_str(&env, "TKNA/TKNB"),
        //&PhoenixPoolType::Constant,
        &phoenix_factory::PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );
    let phoenix_pool_stable_bc_address = phoenix_factory.create_liquidity_pool(
        &admin,
        &pho_bc_lp_init_info,
        &String::from_str(&env, "phoenix_pool_bc"),
        &String::from_str(&env, "TKNB/TKNC"),
        &phoenix_factory::PoolType::Stable,
        &None::<u64>,
        &100i64,
        &1_000,
    );
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
) -> (SoroswapFactoryClient<'a>, SoroswapRouterClient<'a>) {
    let soroswap_factory_id = env.register(SOROSWAP_FACTORY_WASM, ());
    let soroswap_router_id = env.register(SOROSWAP_ROUTER_WASM, ());
    let soroswap_pair_wasm_hash = env.deployer().upload_contract_wasm(SOROSWAP_PAIR_WASM);

    let soroswap_factory = SoroswapFactoryClient::new(env, &soroswap_factory_id);
    let soroswap_router = SoroswapRouterClient::new(env, &soroswap_router_id);

    soroswap_factory.initialize(admin, &soroswap_pair_wasm_hash);
    soroswap_router.initialize(&soroswap_factory_id);

    (soroswap_factory, soroswap_router)
}

pub fn setup_test_accounts(env: &Env) -> (Address, Address) {
    let admin = Address::generate(env);
    let user = Address::generate(env);

    (admin, user)
}

impl<'a> HoopsTestEnvironment<'a> {
    pub fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        env.cost_estimate().budget().reset_unlimited(); // Corrected budget reset

        // Deploy Tokens
        let (admin, user) = setup_test_accounts(&env);

        let (token_a_client, token_a_admin) = create_stellar_token(&env, &admin);
        let (token_b_client, token_b_admin) = create_stellar_token(&env, &admin);
        let (token_c_client, token_c_admin) = create_stellar_token(&env, &admin);

        let initial_mint_amount: i128 = 10_000_000 * 10_000_000; // 10M tokens with 7 decimals

        // mint the initial tokens to admin:
        token_a_admin.mint(&admin, &initial_mint_amount);
        token_b_admin.mint(&admin, &initial_mint_amount);
        token_c_admin.mint(&admin, &initial_mint_amount);

        // Mint tokens to the user as well
        token_a_admin.mint(&user, &initial_mint_amount);
        token_b_admin.mint(&user, &initial_mint_amount);
        token_c_admin.mint(&user, &initial_mint_amount);

        let soroswap_factory_id = env.register(SOROSWAP_FACTORY_WASM, ());
        let soroswap_router_id = env.register(SOROSWAP_ROUTER_WASM, ());
        let soroswap_pair_wasm_hash = env.deployer().upload_contract_wasm(SOROSWAP_PAIR_WASM);

        let soroswap_factory = SoroswapFactoryClient::new(&env, &soroswap_factory_id);
        let soroswap_router = SoroswapRouterClient::new(&env, &soroswap_router_id);

        soroswap_factory.initialize(&admin, &soroswap_pair_wasm_hash);
        soroswap_router.initialize(&soroswap_factory_id);

        let mut soroswap_pool_ids = Vec::new(&env);

        soroswap_factory.create_pair(&token_a_client.address, &token_b_client.address);
        let pair_ab_address =
            soroswap_factory.get_pair(&token_a_client.address, &token_b_client.address);
        soroswap_pool_ids.push_back(pair_ab_address.clone());
        token_a_client.approve(
            &user,
            &soroswap_router.address,
            &(initial_mint_amount / 2),
            &200,
        );
        token_b_client.approve(
            &user,
            &soroswap_router.address,
            &(initial_mint_amount / 2),
            &200,
        );
        soroswap_router.add_liquidity(
            &token_a_client.address,
            &token_b_client.address,
            &(initial_mint_amount / 10), // desired_a
            &(initial_mint_amount / 10), // desired_b
            &0,                          // min_a
            &0,                          // min_b
            &user,
            &(env.ledger().timestamp() + 100),
        );

        // Pool 2: token_b/token_c
        soroswap_factory.create_pair(&token_b_client.address, &token_c_client.address);
        let pair_bc_address =
            soroswap_factory.get_pair(&token_b_client.address, &token_c_client.address); // This line had E0599
        soroswap_pool_ids.push_back(pair_bc_address.clone());
        token_b_client.approve(
            &user,
            &soroswap_router.address,
            &(initial_mint_amount / 2),
            &200,
        );
        token_c_client.approve(
            &user,
            &soroswap_router.address,
            &(initial_mint_amount / 2),
            &200,
        );
        soroswap_router.add_liquidity(
            &token_b_client.address,
            &token_c_client.address,
            &(initial_mint_amount / 10), // desired_a
            &(initial_mint_amount / 10), // desired_b
            &0,                          // min_a
            &0,                          // min_b
            &user,
            &(env.ledger().timestamp() + 100),
        );
        let soroswap_amm = AmmInfrastructure {
            name: "Soroswap".into_val(&env),
            factory_id: soroswap_factory_id,
            router_id: Some(soroswap_router_id.clone()),
            pool_ids: soroswap_pool_ids,
        };

        // --- Aqua Setup --- (this should be it's own function later.)
        let aqua_router_id = env.register(AQUA_ROUTER_WASM, ());
        let aqua_router = AquaRouterClient::new(&env, &aqua_router_id);
        let aqua_pool_wasm_hash = env.deployer().upload_contract_wasm(AQUA_POOL_CONSTANT_WASM);
        let aqua_stable_pool_wasm_hash = env.deployer().upload_contract_wasm(AQUA_STABLE_POOL_WASM);

        aqua_router.init_admin(&admin);
        std::println!("Aqua Router initialized with admin: {:?}", admin);
        let mut aqua_pool_ids = Vec::new(&env);

        let aqua_tokens = Vec::from_array(
            &env,
            [
                token_a_client.address.clone(),
                token_b_client.address.clone(),
            ],
        );

        let (aqua_pool_ab_hash, aqua_pool_ab_address) =
            aqua_router.init_standard_pool(&user, &aqua_tokens, &30);

        // i'm not sure but we may need to track constant and stables seperately.
        aqua_pool_ids.push_back(aqua_pool_ab_address.clone());

        token_a_client.approve(
            &user,
            &aqua_pool_ab_address,
            &(initial_mint_amount / 2),
            &200,
        );
        token_b_client.approve(
            &user,
            &aqua_pool_ab_address,
            &(initial_mint_amount / 2),
            &200,
        );

        let aqua_pool_ab_client = AquaPoolClient::new(&env, &aqua_pool_ab_address);

        // Deposit into Aqua Pool AB
        let deposit_amounts_ab = vec![
            &env,
            initial_mint_amount as u128 / 10,
            initial_mint_amount as u128 / 10,
        ];

        let _aqua_ab_shares = aqua_router.deposit(
            &user,
            &aqua_tokens,
            &aqua_pool_ab_hash,
            &deposit_amounts_ab,
            &0,
        );

        let aqua_ab_liquidity = aqua_router.get_liquidity(&aqua_tokens, &aqua_pool_ab_hash);

        let a_aqua: u128 = 100;
        let fee_fraction_aqua: u32 = 30; // 0.3%

        let aqua_stable_tokens = Vec::from_array(
            &env,
            [
                token_b_client.address.clone(),
                token_c_client.address.clone(),
            ],
        );

        let (aqua_stable_bc_hash, aqua_stable_bc_address) =
            aqua_router.init_stableswap_pool(&user, &aqua_stable_tokens, &10);

        aqua_pool_ids.push_back(aqua_stable_bc_address.clone());

        token_b_client.approve(
            &user,
            &aqua_stable_bc_address,
            &(initial_mint_amount / 2),
            &200,
        );
        token_c_client.approve(
            &user,
            &aqua_stable_bc_address,
            &(initial_mint_amount / 2),
            &200,
        );

        // for testing deposit or swaps direct to pool.
        let aqua_pool_bc_stable_client = AquaStablePoolClient::new(&env, &aqua_stable_bc_address);
        // for testing against the router.
        let deposit_amounts_bc = vec![
            &env,
            initial_mint_amount as u128 / 10,
            initial_mint_amount as u128 / 10,
        ];
        let _shares_aqua_pool_bc =
            aqua_pool_bc_stable_client.deposit(&user, &deposit_amounts_bc, &0);
        let _shares_aqua_router_bc = aqua_router.deposit(
            &user,
            &aqua_stable_tokens,
            &aqua_stable_bc_hash,
            &deposit_amounts_bc,
            &0,
        );

        let aqua_amm = AmmInfrastructure {
            name: "Aqua".into_val(&env),
            factory_id: aqua_router_id.clone(),
            router_id: Some(aqua_router_id.clone()),
            pool_ids: aqua_pool_ids,
        };

        let phoenix_factory = deploy_phoenix_factory_contract(&env, Some(admin.clone()));
        let phoenix_factory_id = phoenix_factory.address.clone();
        let phoenix_amm = set_phoenix_amm_infra(
            &env,
            admin.clone(),
            user.clone(),
            &phoenix_factory,
            &token_a_client,
            &token_b_client,
            &token_c_client,
        );

        // --- Comet Setup ---yea
        let comet_factory_id = env.register(COMET_FACTORY_WASM, ());
        let comet_factory = CometFactoryClient::new(&env, &comet_factory_id);
        let comet_pool_wasm_hash = env.deployer().upload_contract_wasm(COMET_POOL_WASM);
        comet_factory.init(&comet_pool_wasm_hash);
        let mut comet_pool_ids = Vec::new(&env);

        let comet_amm = AmmInfrastructure {
            name: "Comet".into_val(&env),
            factory_id: comet_factory_id.clone(),
            router_id: None,
            pool_ids: comet_pool_ids, // Will be empty for now
        };

        // --- Deploy Adapters ---
        let soroswap_adapter_id = env.register(SOROSWAP_ADAPTER_WASM, ());
        let soroswap_adapter = SoroswapAdapterClient::new(&env, &soroswap_adapter_id);
        soroswap_adapter.initialize(&3, &soroswap_router_id);

        let aqua_adapter_id = env.register(AQUA_ADAPTER_WASM, ());
        let aqua_adapter = AquaAdapterClient::new(&env, &aqua_adapter_id);
        aqua_adapter.initialize(&0, &aqua_router_id);

        let phoenix_adapter_id = env.register(PHOENIX_ADAPTER_WASM, ());
        let phoenix_adapter = PhoenixAdapterClient::new(&env, &phoenix_adapter_id);

        if let Some(first_phoenix_pool) = phoenix_amm.pool_ids.get(0) {
            phoenix_adapter.initialize(&2, &first_phoenix_pool);
        } else {
            phoenix_adapter.initialize(&2, &phoenix_factory_id);
        }

        let comet_adapter_id = env.register(COMET_ADAPTER_WASM, ());
        let comet_adapter = CometAdapterClient::new(&env, &comet_adapter_id);
        if let Some(first_comet_pool) = comet_amm.pool_ids.get(0) {
            comet_adapter.initialize(&1, &first_comet_pool);
        } else {
            comet_adapter.initialize(&1, &comet_factory_id);
        }

        let adapters = AdapterContracts {
            soroswap: soroswap_adapter,
            aqua: aqua_adapter,
            phoenix: phoenix_adapter,
            comet: comet_adapter,
        };

        // --- Deploy Hoops Router ---
        let hoops_router_id = env.register(HOOPS_ROUTER_WASM, ());
        let hoops_router = HoopsRouterClient::new(&env, &hoops_router_id);

        /*
        let mut adapter_configs = Vec::new(&env);
        adapter_configs.push_back((3u32, soroswap_adapter_id.clone())); // Ensure protocol_id is u32
        adapter_configs.push_back((0u32, aqua_adapter_id.clone()));
        adapter_configs.push_back((2u32, phoenix_adapter_id.clone()));
        adapter_configs.push_back((1u32, comet_adapter_id.clone()));
*/
        hoops_router.initialize(&admin);

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

#[test]
fn test_environment_setup_placeholders() {
    let _test_env = HoopsTestEnvironment::setup();
    
}
