pub mod phoenix_pair {
    soroban_sdk::contractimport!(
        file = "../../bytecodes/phoenix_pool.wasm"
    );
    pub type PhoenixPoolClient<'a> = Client<'a>;
}

pub mod phoenix_factory {
    soroban_sdk::contractimport!(
        file = "../../bytecodes/phoenix_factory.wasm"
    );
    pub type PhoenixFactoryClient<'a> = Client<'a>;
}
