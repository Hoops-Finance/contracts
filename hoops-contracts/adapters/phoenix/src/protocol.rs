pub mod phoenix_pair {
    soroban_sdk::contractimport!(
        file = "../../bytecodes/phoenix_pool.wasm"
    );
    pub type PhoenixPoolClient<'a> = Client<'a>;
}
