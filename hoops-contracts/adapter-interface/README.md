# Hoops Adapter Interface Crate

This crate defines the standardized interface for AMM (Automated Market Maker) adapters within the Hoops Finance ecosystem.

## Purpose

The `hoops-adapter-interface` crate provides the core `AdapterTrait`. All AMM adapter contracts (e.g., for Soroswap, Phoenix, etc.) must implement this trait. This ensures that the `Router` contract can interact with any supported AMM in a consistent and predictable manner, abstracting away the specific implementation details of each protocol.

## Key Components

### Error Types

*   **`AdapterError`**: An enum defining errors that can originate from AMM adapter operations.
    *   `DefaultError`: A generic error category.
    *   `UnsupportedPair`: Indicates that a token pair is not supported by the adapter or AMM.
    *   `ExternalFailure`: Signals an error originating from the underlying external AMM contract.

    *(Note: A similar `AdapterError` is also defined in the `hoops-common` crate. This duplication needs to be resolved.)*

### Traits and Clients

*   **`AdapterTrait`**: Defines the standardized interface that all AMM adapters MUST implement.
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

*   **`AdapterClient`**: A Soroban contract client generated for the `AdapterTrait`, allowing other contracts to easily call its functions.

## Usage

This crate is intended to be used as a dependency by:
*   All individual AMM adapter contracts (which will implement `AdapterTrait`).
*   The `Router` contract (which will use `AdapterClient` to call adapter functions).

## TODO

*   **Resolve Duplication with `hoops-common`**: The `AdapterTrait`, `AdapterClient`, and `AdapterError` are currently defined in both `hoops-common` and `hoops-adapter-interface`. This is a critical issue to resolve. The recommended approach is:
    *   Define the `AdapterTrait` and `AdapterClient` *only* in `hoops-adapter-interface`.
    *   Define shared error types like `AdapterError` (or a more generic `HoopsError`) *only* in `hoops-common`.
    *   Ensure `hoops-adapter-interface` depends on `hoops-common` for error types, and other crates depend on `hoops-adapter-interface` for the trait and client.
*   **Refine `AdapterError`**: Once duplication is resolved, ensure the `AdapterError` enum in `hoops-common` is comprehensive enough for all adapter needs, or introduce more specific error types if necessary.
*   **Consider Advanced Trait Features**: Evaluate if the `AdapterTrait` needs to be extended with more advanced features, such as:
    *   Functions to query pool reserves or prices.
    *   Support for multi-hop swaps directly within the adapter interface (if not handled solely by the Router).
    *   More granular options for liquidity provision (e.g., single-sided, specific price ranges for concentrated liquidity AMMs).
