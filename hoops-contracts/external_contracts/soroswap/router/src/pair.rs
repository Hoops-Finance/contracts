soroban_sdk::contractimport!(
    file = "../../../bytecodes/soroswap_pair.wasm"
);
pub type SoroswapPairClient<'a> = Client<'a>;