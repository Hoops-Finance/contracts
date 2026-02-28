pub mod soroswap_router {
soroban_sdk::contractimport!(
    file = "../../bytecodes/soroswap_router.wasm"
);
pub type SoroswapRouterClient<'a> = Client<'a>;
}

pub mod soroswap_pair {
    soroban_sdk::contractimport!(
    file = "../../bytecodes/soroswap_pair.wasm"
);
pub type SoroswapPairClient<'a> = Client<'a>;
}

pub mod soroswap_factory {
    soroban_sdk::contractimport!(
        file = "../../bytecodes/soroswap_factory.wasm"
    );
    pub type SoroswapFactoryClient<'a> = Client<'a>;
}
