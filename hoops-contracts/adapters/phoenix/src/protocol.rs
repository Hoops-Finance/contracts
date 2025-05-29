pub mod phoenix_pair {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/phoenix_pool.wasm"
    );
    pub type PhoenixPoolClient<'a> = Client<'a>;
}
