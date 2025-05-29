#![no_std]
mod contract;
mod error;
mod storage;
#[cfg(test)]
mod tests;

pub mod token_contract {
    soroban_sdk::contractimport!(
        file = "../../../target/wasm32v1-none/release/soroban_token_contract.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub mod lp_contract {
    soroban_sdk::contractimport!(
        file = "../../../target/wasm32v1-none/release/phoenix_pool.wasm"
    );
}
