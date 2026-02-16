use soroban_sdk::{
    Env,
    Address,
    BytesN,
    Vec,
    testutils::Address as _,
    token,
};
use crate::{Account, AccountClient, AccountError, LpPlan};

// Import the router contract
pub mod hoops_router {
    soroban_sdk::contractimport!(
        file = "../bytecodes/hoops_router.wasm"
    );
    pub type RouterClient<'a> = Client<'a>;
}
use hoops_router::RouterClient;

const DECIMALS: u32 = 7;
const TOKEN_UNIT: i128 = 10i128.pow(DECIMALS);

struct TestEnv {
    env: Env,
    #[allow(dead_code)]
    admin: Address,
    user: Address,
    account_contract_id: Address,
    account_client: AccountClient<'static>,
    router_contract_id: Address,
    #[allow(dead_code)]
    router_client: RouterClient<'static>,
    usdc_token_id: Address,
    usdc_token_client: token::StellarAssetClient<'static>,
    usdc_balance_client: token::Client<'static>,
    lp_token_id: Address,
    lp_token_client: token::StellarAssetClient<'static>,
    lp_balance_client: token::Client<'static>,
}

impl TestEnv {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        // Deploy and initialize Account contract
        let account_contract_id = env.register(Account, ());
        let account_client = AccountClient::new(&env, &account_contract_id);

        // Deploy Router contract
        let router_contract_id = env.register_contract_wasm(None, hoops_router::WASM);
        let router_client = RouterClient::new(&env, &router_contract_id);

        // Deploy Token contracts (StellarAssetClient for admin ops like mint)
        let usdc_issuer = Address::generate(&env);
        let usdc_token_id = env.register_stellar_asset_contract_v2(usdc_issuer);
        let usdc_token_client = token::StellarAssetClient::new(&env, &usdc_token_id.address());
        let usdc_balance_client = token::Client::new(&env, &usdc_token_id.address());

        let lp_issuer = Address::generate(&env);
        let lp_token_id = env.register_stellar_asset_contract_v2(lp_issuer);
        let lp_token_client = token::StellarAssetClient::new(&env, &lp_token_id.address());
        let lp_balance_client = token::Client::new(&env, &lp_token_id.address());

        // Initialize account contract
        account_client.initialize(&user, &router_contract_id);

        // Mint some tokens to the user for testing
        usdc_token_client.mint(&user, &(1000 * TOKEN_UNIT));

        TestEnv {
            env,
            admin,
            user,
            account_contract_id,
            account_client,
            router_contract_id,
            router_client,
            usdc_token_id: usdc_token_id.address(),
            usdc_token_client,
            usdc_balance_client,
            lp_token_id: lp_token_id.address(),
            lp_token_client,
            lp_balance_client,
        }
    }
}

#[test]
fn test_initialize() {
    let TestEnv { user, account_client, router_contract_id, .. } = TestEnv::setup();

    assert_eq!(account_client.owner(), user);
    assert_eq!(account_client.router(), router_contract_id);

    // Try to initialize again, should fail
    let init_again_result = account_client.try_initialize(&user, &router_contract_id);
    assert_eq!(init_again_result, Err(Ok(AccountError::AlreadyInitialized)));
}

#[test]
fn test_transfer() {
    let TestEnv { env, account_client, usdc_token_client, usdc_balance_client, usdc_token_id, account_contract_id, .. } = TestEnv::setup();
    let recipient = Address::generate(&env);
    let transfer_amount = 100 * TOKEN_UNIT;

    // Mint some USDC to the account contract for it to transfer
    usdc_token_client.mint(&account_contract_id, &(200 * TOKEN_UNIT));
    assert_eq!(usdc_balance_client.balance(&account_contract_id), 200 * TOKEN_UNIT);

    // Owner initiates transfer
    account_client.transfer(&usdc_token_id, &recipient, &transfer_amount);

    assert_eq!(usdc_balance_client.balance(&account_contract_id), 100 * TOKEN_UNIT);
    assert_eq!(usdc_balance_client.balance(&recipient), transfer_amount);
}

// ---- Passkey tests ----

#[test]
fn test_set_passkey_pubkey() {
    let TestEnv { env, account_client, .. } = TestEnv::setup();

    // Generate a fake 65-byte uncompressed secp256r1 public key
    let pubkey = BytesN::from_array(&env, &[4u8; 65]);

    // Set passkey (first time, no auth required)
    account_client.set_passkey_pubkey(&pubkey);

    // Retrieve and verify
    let stored = account_client.get_passkey_pubkey();
    assert_eq!(stored, Some(pubkey));
}

#[test]
fn test_get_passkey_pubkey_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let account_id = env.register(Account, ());
    let client = AccountClient::new(&env, &account_id);
    let user = Address::generate(&env);
    let router = Address::generate(&env);

    // Initialize without passkey
    client.initialize(&user, &router);

    // No passkey set → returns None
    assert_eq!(client.get_passkey_pubkey(), None);
}

#[test]
fn test_initialize_with_passkey() {
    let env = Env::default();
    env.mock_all_auths();

    let account_id = env.register(Account, ());
    let client = AccountClient::new(&env, &account_id);
    let user = Address::generate(&env);
    let router = Address::generate(&env);
    let pubkey = BytesN::from_array(&env, &[4u8; 65]);

    // Initialize with passkey in one call
    client.initialize_with_passkey(&user, &router, &pubkey);

    // Verify all three fields are stored
    assert_eq!(client.owner(), user);
    assert_eq!(client.router(), router);
    assert_eq!(client.get_passkey_pubkey(), Some(pubkey));
}

#[test]
fn test_initialize_with_passkey_rejects_double_init() {
    let env = Env::default();
    env.mock_all_auths();

    let account_id = env.register(Account, ());
    let client = AccountClient::new(&env, &account_id);
    let user = Address::generate(&env);
    let router = Address::generate(&env);
    let pubkey = BytesN::from_array(&env, &[4u8; 65]);

    client.initialize_with_passkey(&user, &router, &pubkey);

    // Second initialization should fail
    let result = client.try_initialize_with_passkey(&user, &router, &pubkey);
    assert_eq!(result, Err(Ok(AccountError::AlreadyInitialized)));
}

#[test]
fn test_regular_init_blocks_passkey_init() {
    let TestEnv { account_client, user, router_contract_id, env, .. } = TestEnv::setup();

    let pubkey = BytesN::from_array(&env, &[4u8; 65]);

    // Already initialized via regular init in setup — passkey init should fail
    let result = account_client.try_initialize_with_passkey(&user, &router_contract_id, &pubkey);
    assert_eq!(result, Err(Ok(AccountError::AlreadyInitialized)));
}
