pub mod aqua_liquidity_pool {
    soroban_sdk::contractimport!(file = "../../bytecodes/aqua_soroban_liquidity_pool_contract.wasm");
    pub type AquaPoolClient<'a> = Client<'a>;
    }
pub use aqua_liquidity_pool::AquaPoolClient;
/*
pub mod aqua_router {

    soroban_sdk::contractimport!(file = "../../bytecodes/aqua_liquidity_pool_router_contract.wasm");
    pub type AquaRouterClient<'a> = Client<'a>;
}
pub use aqua_router::AquaRouterClient;
pub mod aqua_pool {
    soroban_sdk::contractimport!(file = "../../bytecodes/aqua_soroban_liquidity_pool_contract.wasm");
    pub type AquaConstantProductPoolClient<'a> = Client<'a>;
}
pub use aqua_pool::AquaConstantProductPoolClient;
pub mod aqua_stableswap_pool {
    soroban_sdk::contractimport!(file = "../../bytecodes/aqua_soroban_liquidity_pool_stableswap_contract.wasm");
    pub type AquaStableSwapPoolClient<'a> = Client<'a>;
}
pub use aqua_stableswap_pool::AquaStableSwapPoolClient;
*/