#![no_std]
mod contract;
mod distribution;
mod error;
mod msg;
mod storage;

pub const TOKEN_PER_POWER: i32 = 1_000;

pub mod token_contract {
    // The import will code generate:
    // - A ContractClient type that can be used to invoke functions on the contract.
    // - Any types in the contract that were annotated with #[contracttype].
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/soroban_token_contract_phoenix.wasm"
    );
}

#[cfg(test)]
mod phoenix_stake_tests;
