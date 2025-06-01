soroban_sdk::contractimport!(
    file = "../../bytecodes/aqua_liquidity_pool_router_contract.wasm" // aquas router.
);
pub type AquaRouterClient<'a> = Client<'a>;
