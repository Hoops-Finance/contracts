soroban_sdk::contractimport!(
    file = "../../target/wasm32v1-none/release/comet-pool.wasm"
);
pub type CometPoolClient<'a> = Client<'a>;
