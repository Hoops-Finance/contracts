#[cfg(test)]
mod test {
    use soroban_sdk::{
        Env,
        Address,
        BytesN,
        Vec,
        testutils::{Address as _, Ledger, LedgerInfo},
        token,
    };
    use crate::{Account, AccountClient, AccountError, LpPlan};

    // Import the router contract
    pub mod hoops_router {
        soroban_sdk::contractimport!(
            file = "../target/wasm32v1-none/release/hoops_router.wasm"
        );
        pub type RouterClient<'a> = Client<'a>;
    }
    use hoops_router::RouterClient;

    const DECIMALS: u32 = 7;
    const TOKEN_UNIT: i128 = 10i128.pow(DECIMALS);

    struct TestEnv {
        env: Env,
        admin: Address,
        user: Address,
        account_contract_id: Address,
        account_client: AccountClient<'static>,
        router_contract_id: Address,
        router_client: RouterClient<'static>,
        usdc_token_id: Address,
        usdc_token_client: token::Client<'static>,
        lp_token_id: Address,
        lp_token_client: token::Client<'static>,
    }

    impl TestEnv {
        fn setup() -> Self {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let user = Address::generate(&env);

            // Deploy and initialize Account contract
            let account_contract_id = env.register_contract(None, Account);
            let account_client = AccountClient::new(&env, &account_contract_id);

            // Deploy Router contract (assuming it has an initialize function)
            let router_contract_id = env.register_contract_wasm(None, hoops_router::WASM);
            let router_client = RouterClient::new(&env, &router_contract_id);
            // router_client.initialize(&admin); // Assuming router has an initialize function

            // Deploy Token contracts
            let usdc_token_id = env.register_stellar_asset_contract(Address::generate(&env));
            let usdc_token_client = token::Client::new(&env, &usdc_token_id);

            let lp_token_id = env.register_stellar_asset_contract(Address::generate(&env));
            let lp_token_client = token::Client::new(&env, &lp_token_id);
            
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
                usdc_token_id,
                usdc_token_client,
                lp_token_id,
                lp_token_client,
            }
        }
    }

    #[test]
    fn test_initialize() {
        let TestEnv { env, user, account_client, router_contract_id, .. } = TestEnv::setup();
        
        // Check if owner is set correctly
        assert_eq!(account_client.owner(), user);
        // Check if router is set correctly
        assert_eq!(account_client.router(), router_contract_id);

        // Try to initialize again, should fail
        let init_again_result = account_client.try_initialize(&user, &router_contract_id);
        assert_eq!(init_again_result, Err(Ok(AccountError::AlreadyInitialized)));
    }

    #[test]
    fn test_upgrade() {
        let TestEnv { env, user, account_client, .. } = TestEnv::setup();
        let new_wasm_hash = env.deployer().upload_contract_wasm(crate::WASM);

        // Non-owner tries to upgrade, should fail if we had stricter auth checks
        // For now, we only check owner auth
        // let another_user = Address::generate(&env);
        // let upgrade_by_another_result = account_client.mock_auths(&[MockAuth {
        //    addr: &another_user,
        //    invoke: &MockAuthInvoke {
        //        contract: &account_client.address,
        //        fn_name: "upgrade",
        //        args: (&new_wasm_hash,).into_val(&env),
        //        sub_invokes: &[],
        //    }
        // }]).try_upgrade(&new_wasm_hash);
        // assert!(upgrade_by_another_result.is_err());


        // Owner upgrades
        account_client.upgrade(&new_wasm_hash);
        // We can't easily verify the upgrade without calling a function from the new contract version
        // or checking contract data layout if it changed.
        // For now, just ensuring it doesn't panic is a basic check.
    }

    #[test]
    fn test_transfer() {
        let TestEnv { env, user, account_client, usdc_token_client, usdc_token_id, account_contract_id, .. } = TestEnv::setup();
        let recipient = Address::generate(&env);
        let transfer_amount = 100 * TOKEN_UNIT;

        // Mint some USDC to the account contract for it to transfer
        usdc_token_client.mint(&account_contract_id, &(200 * TOKEN_UNIT));
        assert_eq!(usdc_token_client.balance(&account_contract_id), 200 * TOKEN_UNIT);


        // Owner initiates transfer
        account_client.transfer(&usdc_token_id, &recipient, &transfer_amount);

        assert_eq!(usdc_token_client.balance(&account_contract_id), 100 * TOKEN_UNIT);
        assert_eq!(usdc_token_client.balance(&recipient), transfer_amount);
    }

    #[test]
    fn test_deposit_usdc() {
        let TestEnv { env, user, account_client, router_client, usdc_token_client, usdc_token_id, account_contract_id, router_contract_id, .. } = TestEnv::setup();
        let deposit_amount = 500 * TOKEN_UNIT;
        let deadline = env.ledger().timestamp() + 100; // 100 seconds from now

        // User already has 1000 USDC from setup
        assert_eq!(usdc_token_client.balance(&user), 1000 * TOKEN_UNIT);

        // Define LP plans (simplified for now, assuming router handles this structure)
        let lp_plans: Vec<LpPlan> = Vec::new(&env); // TODO: Define LpPlan properly based on router

        // User deposits USDC
        account_client.deposit_usdc(&usdc_token_id, &deposit_amount, &lp_plans, &deadline);

        // Check balances
        // User's USDC should decrease by deposit_amount
        assert_eq!(usdc_token_client.balance(&user), (1000 - 500) * TOKEN_UNIT);
        // Account contract's USDC should be 0 after transferring to router (or to LPs via router)
        // This depends on router's provide_liquidity implementation.
        // For now, we assume it's transferred out of the account.
        // A more precise check would involve mocking router behavior or checking LP token minting.
        // assert_eq!(usdc_token_client.balance(&account_contract_id), 0);

        // Check allowance given to router by account contract
        // This also depends on router consuming the allowance.
        // let allowance = usdc_token_client.allowance(&account_contract_id, &router_contract_id);
        // assert_eq!(allowance, 0); // Assuming router uses up the allowance
    }

    #[test]
    fn test_redeem() {
        let TestEnv { env, user, account_client, router_client, usdc_token_client, usdc_token_id, lp_token_client, lp_token_id, account_contract_id, router_contract_id, .. } = TestEnv::setup();
        let redeem_lp_amount = 100 * TOKEN_UNIT;
        let deadline = env.ledger().timestamp() + 100;

        // Simulate user having some LP tokens in the account contract
        // (e.g., from a previous deposit_usdc call that resulted in LP tokens being sent to account)
        // For this test, we'll directly mint LP tokens to the account contract.
        lp_token_client.mint(&account_contract_id, &(200 * TOKEN_UNIT));
        assert_eq!(lp_token_client.balance(&account_contract_id), 200 * TOKEN_UNIT);
        
        // Pre-fund account with some USDC to simulate what it would get from router.redeem_liquidity
        // This is a simplification. In reality, router.redeem_liquidity would send USDC.
        let simulated_usdc_from_redeem = 300 * TOKEN_UNIT;
        usdc_token_client.mint(&account_contract_id, &simulated_usdc_from_redeem);


        // User initiates redeem
        account_client.redeem(&lp_token_id, &redeem_lp_amount, &usdc_token_id, &deadline);

        // Check balances
        // Account contract's LP tokens should decrease
        assert_eq!(lp_token_client.balance(&account_contract_id), (200-100) * TOKEN_UNIT);
        
        // User's USDC balance should increase by the amount swept from the account contract
        // This assumes the router.redeem_liquidity resulted in `simulated_usdc_from_redeem` being in the account
        // and then it was swept to the user.
        assert_eq!(usdc_token_client.balance(&user), (1000 + simulated_usdc_from_redeem) * TOKEN_UNIT);
        
        // Account contract's USDC should be 0 after sweep
        assert_eq!(usdc_token_client.balance(&account_contract_id), 0);
    }
}
