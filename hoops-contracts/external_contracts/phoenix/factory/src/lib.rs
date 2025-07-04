#![no_std]
mod contract;
mod error;
mod storage;
mod utils;

#[cfg(test)]
mod pho_factory_tests;

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

pub trait ConvertVec<T, U> {
    fn convert_vec(&self) -> soroban_sdk::Vec<U>;
}

impl ConvertVec<stake_contract::Stake, storage::Stake> for soroban_sdk::Vec<stake_contract::Stake> {
    fn convert_vec(&self) -> soroban_sdk::Vec<storage::Stake> {
        let env = self.env(); // Get the environment
        let mut result = soroban_sdk::Vec::new(env);

        for stake in self.iter() {
            result.push_back(storage::Stake {
                stake: stake.stake,
                stake_timestamp: stake.stake_timestamp,
            });
        }

        result
    }
}
