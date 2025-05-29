pub mod soroswap_router {
soroban_sdk::contractimport!(
    file = "../../target/wasm32v1-none/release/soroswap_router.wasm"
);
pub type SoroswapRouterClient<'a> = Client<'a>;
} 

pub mod soroswap_pair {
    soroban_sdk::contractimport!(
    file = "../../target/wasm32v1-none/release/soroswap_pair.wasm"
);
pub type SoroswapPairClient<'a> = Client<'a>;
}
