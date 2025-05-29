soroban_sdk::contractimport!(
    file = "../../target/wasm32v1-none/release/aqua_liquidity_pool_router_contract.wasm" // aquas router.
);
pub type AquaRouterClient<'a> = Client<'a>;
