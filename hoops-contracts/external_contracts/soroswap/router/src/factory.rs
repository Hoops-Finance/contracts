soroban_sdk::contractimport!(
    file = "../../../target/wasm32v1-none/release/soroswap_factory.wasm"
);
pub type SoroswapFactoryClient<'a> = Client<'a>;