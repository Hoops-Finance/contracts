#![no_std]
mod contract;
mod error;
mod math;
mod storage;

pub mod token_contract {
    // The import will code generate:
    // - A ContractClient type that can be used to invoke functions on the contract.
    // - Any types in the contract that were annotated with #[contracttype].
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/soroban_token_contract_phoenix.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub mod stake_contract {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/phoenix_stake.wasm"
    );
}

const DECIMAL_PRECISION: u32 = 18;

#[cfg(test)]
mod phoenix_pool_stable_tests;
