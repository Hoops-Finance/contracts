#![no_std]
mod contract;
mod error;
mod storage;
#[cfg(test)]
mod phoenix_trader_tests;

pub mod token_contract {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/soroban_token_contract_phoenix.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub mod lp_contract {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/phoenix_pool.wasm"
    );
}
