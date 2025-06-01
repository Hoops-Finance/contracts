soroban_sdk::contractimport!(
    file = "../../../bytecodes/soroswap_factory.wasm"
);
pub type SoroswapFactoryClient<'a> = Client<'a>;