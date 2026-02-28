soroban_sdk::contractimport!(
    file = "../../bytecodes/comet-pool.wasm"
);
pub type CometPoolClient<'a> = Client<'a>;

pub mod comet_factory {
    soroban_sdk::contractimport!(
        file = "../../bytecodes/comet_factory.wasm"
    );
    pub type CometFactoryClient<'a> = Client<'a>;
}
