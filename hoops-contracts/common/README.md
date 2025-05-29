# Hoops Common Crate

This crate provides common types and definitions used across the Hoops Finance ecosystem.

## Purpose

The `hoops-common` crate serves as a foundational library for other Hoops contracts, ensuring consistency and reusability of core components.

## Key Components

### Error Types

*   **`AdapterError`**: An enum defining errors that can originate from AMM adapter operations.
    *   `Common`: A generic error category.
    *   `UnsupportedPair`: Indicates that a token pair is not supported by the adapter or AMM.
    *   `ExternalFailure`: Signals an error originating from the underlying external AMM contract.

### Traits and Clients (Currently defined in this crate)

*   **`Adapter` Trait**: Defines the standardized interface that all Automated Market Maker (AMM) adapters within the Hoops Finance system MUST implement. This allows the `Router` contract to interact with different AMMs in a uniform way.
    *   **Lifecycle Functions:**
        *   `initialize(e: Env, amm_id: i128, amm_address: Address) -> Result<(), AdapterError>`: Initializes the adapter with the specific AMM's identifier and contract address.
        *   `upgrade(e: Env, new_wasm: BytesN<32>) -> Result<(), AdapterError>`: Upgrades the adapter contract to a new WASM hash.
        *   `version() -> u32`: Returns the current version of the adapter.
    *   **Swap Functions:**
        *   `swap_exact_in(...) -> Result<i128, AdapterError>`: Swaps an exact amount of an input token for a minimum amount of an output token.
        *   `swap_exact_out(...) -> Result<i128, AdapterError>`: Swaps a maximum amount of an input token for an exact amount of an output token.
    *   **Liquidity Functions:**
        *   `add_liquidity(...) -> Result<Address, AdapterError>`: Adds liquidity to an AMM pool and returns the address of the LP token.
        *   `remove_liquidity(...) -> Result<(i128,i128), AdapterError>`: Removes liquidity from an AMM pool and returns the amounts of the withdrawn tokens.

*   **`AdapterClient`**: A Soroban contract client generated for the `Adapter` trait, allowing other contracts to easily call its functions.

## Usage

This crate is intended to be included as a dependency by other Hoops Finance contracts.

## TODO

*   **Refactor Trait Location**: Consider moving the `Adapter` trait and its associated `AdapterClient` to the `hoops-contracts/adapter-interface` crate. This would provide a clearer separation of concerns, with `hoops-common` focusing solely on shared data types (like `AdapterError`) and `hoops-adapter-interface` defining the core interaction contract for adapters.
*   **Expand Common Errors**: Evaluate if more granular common error types are needed as the project evolves, potentially creating a `CoreError` enum for non-adapter-specific issues.
