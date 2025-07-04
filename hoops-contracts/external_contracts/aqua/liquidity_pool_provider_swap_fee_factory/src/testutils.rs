#![cfg(test)]
extern crate std;
use crate::ProviderSwapFeeFactoryClient;
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};

pub(crate) struct TestConfig {}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {}
    }
}

#[allow(dead_code)]
pub(crate) struct Setup<'a> {
    pub(crate) env: Env,
    pub(crate) admin: Address,
    pub(crate) emergency_admin: Address,
    pub(crate) contract: ProviderSwapFeeFactoryClient<'a>,
    pub(crate) router: swap_router::Client<'a>,
    pub(crate) token_a: SorobanTokenClient<'a>,
    pub(crate) token_a_admin_client: SorobanTokenAdminClient<'a>,
    pub(crate) token_b: SorobanTokenClient<'a>,

    pub(crate) token_b_admin_client: SorobanTokenAdminClient<'a>,
}

impl Default for Setup<'_> {
    // Create setup from default config
    fn default() -> Self {
        let default_config = TestConfig::default();
        Self::new_with_config(&default_config)
    }
}

impl Setup<'_> {
    pub(crate) fn new_with_config(config: &TestConfig) -> Self {
        let setup = Self::setup(config);
        setup
    }

    pub(crate) fn setup(_config: &TestConfig) -> Self {
        let e: Env = Env::default();
        e.mock_all_auths();
        e.cost_estimate().budget().reset_unlimited();

        let admin = Address::generate(&e);
        let emergency_admin = Address::generate(&e);

        let token_a = create_token_contract(&e, &admin);
        let token_b = create_token_contract(&e, &admin);

        let token_a_admin_client = get_token_admin_client(&e, &token_a.address.clone());
        let token_b_admin_client = get_token_admin_client(&e, &token_b.address.clone());

        // init swap router with all it's complexity
        let pool_hash = install_liq_pool_hash(&e);
        let token_hash = install_token_wasm(&e);
        let plane = deploy_plane_contract(&e);
        let boost_feed = create_reward_boost_feed_contract(&e, &admin);
        let router = deploy_liqpool_router_contract(e.clone());
        router.init_admin(&admin);
        router.set_pool_hash(&admin, &pool_hash);
        router.set_stableswap_pool_hash(&admin, &install_stableswap_liq_pool_hash(&e));
        router.set_token_hash(&admin, &token_hash);
        router.set_reward_token(&admin, &token_a.address);
        router.set_pools_plane(&admin, &plane);
        router.configure_init_pool_payment(
            &admin,
            &token_a.address,
            &10_0000000,
            &1_0000000,
            &router.address,
        );
        router.set_reward_boost_config(&admin, &token_a.address, &boost_feed.address);

        // create swap pool & deposit initial liquidity
        token_a_admin_client.mint(&admin, &10_0000000);
        let (_, pool_address) = router.init_standard_pool(
            &admin,
            &Vec::from_array(&e, [token_a.address.clone(), token_b.address.clone()]),
            &30,
        );
        let swap_pool = liquidity_pool::Client::new(&e, &pool_address);
        token_a_admin_client.mint(&admin, &1_000_000_000_0000000);
        token_b_admin_client.mint(&admin, &1_000_000_000_0000000);
        swap_pool.deposit(
            &admin,
            &Vec::from_array(&e, [1_000_000_000_0000000, 1_000_000_000_0000000]),
            &1,
        );

        let contract = create_contract(
            &e,
            &router.address,
            &admin,
            &emergency_admin,
            &install_swap_fee_collector_wasm(&e),
        );

        Self {
            env: e,
            admin,
            emergency_admin,
            contract,
            router,
            token_a,
            token_a_admin_client,
            token_b,
            token_b_admin_client,
        }
    }
}

pub(crate) fn create_token_contract<'a>(e: &Env, admin: &Address) -> SorobanTokenClient<'a> {
    SorobanTokenClient::new(
        e,
        &e.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

pub mod liquidity_pool {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/aqua_liquidity_pool_contract.wasm"
    );
}

pub(crate) fn get_token_admin_client<'a>(
    e: &Env,
    address: &Address,
) -> SorobanTokenAdminClient<'a> {
    SorobanTokenAdminClient::new(e, address)
}

pub fn create_contract<'a>(
    e: &Env,
    router: &Address,
    admin: &Address,
    emergency_admin: &Address,
    fee_collector_wasm: &BytesN<32>,
) -> ProviderSwapFeeFactoryClient<'a> {
    let contract = ProviderSwapFeeFactoryClient::new(
        e,
        &e.register(
            crate::ProviderSwapFeeFactory,
            (admin, emergency_admin, router, fee_collector_wasm),
        ),
    );
    contract
}

pub mod swap_router {
    soroban_sdk::contractimport!(
        file =
            "../../../bytecodes/aqua_liquidity_pool_router_contract.wasm"
    );
}

fn deploy_liqpool_router_contract<'a>(e: Env) -> swap_router::Client<'a> {
    swap_router::Client::new(&e, &e.register(swap_router::WASM, ()))
}

fn install_token_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/soroban_token_contract.wasm"
    );
    e.deployer().upload_contract_wasm(WASM)
}

fn install_liq_pool_hash(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/aqua_liquidity_pool_contract.wasm"
    );
    e.deployer().upload_contract_wasm(WASM)
}

fn install_stableswap_liq_pool_hash(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/aqua_liquidity_pool_stableswap_contract.wasm"
    );
    e.deployer().upload_contract_wasm(WASM)
}

fn deploy_plane_contract<'a>(e: &Env) -> Address {
    soroban_sdk::contractimport!(
        file =
            "../../../bytecodes/aqua_liquidity_pool_plane_contract.wasm"
    );
    Client::new(e, &e.register(WASM, ())).address
}

mod reward_boost_feed {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/aqua_locker_feed_contract.wasm"
    );
}

pub(crate) mod swap_fee_collector {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/aqua_liquidity_pool_provider_swap_fee_contract.wasm"
    );
}

pub(crate) fn install_swap_fee_collector_wasm(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(swap_fee_collector::WASM)
}

pub(crate) fn create_reward_boost_feed_contract<'a>(
    e: &Env,
    admin: &Address,
) -> reward_boost_feed::Client<'a> {
    reward_boost_feed::Client::new(
        e,
        &e.register(
            reward_boost_feed::WASM,
            reward_boost_feed::Args::__constructor(admin, admin, admin),
        ),
    )
}
