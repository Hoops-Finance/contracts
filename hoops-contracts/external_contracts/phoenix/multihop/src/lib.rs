#![no_std]
mod contract;
mod error;
mod storage;
mod utils;

#[allow(clippy::too_many_arguments)]
pub mod xyk_pool {
    // The import will code generate:
    // - A ContractClient type that can be used to invoke functions on the contract.
    // - Any types in the contract that were annotated with #[contracttype].
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/phoenix_pool.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub mod stable_pool {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/phoenix_pool_stable.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub mod factory_contract {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/phoenix_factory.wasm"
    );
}

pub mod token_contract {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/soroban_token_contract_phoenix.wasm"
    );
}

#[cfg(test)]
mod phoenix_multihop_tests;
