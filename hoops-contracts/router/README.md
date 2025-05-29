# Hoops Router Contract

This contract is the central hub for routing swaps and liquidity operations across various Automated Market Makers (AMMs) integrated into the Hoops Finance ecosystem.

## Purpose

The `Router` contract simplifies user interaction with multiple AMMs by providing a single, unified interface. It maintains a list of registered AMM adapters and delegates operations to the appropriate adapter based on user requests or internal logic.

## Key Components

### Structs and Enums

*   **`RouterError`**: Defines errors specific to router operations:
    *   `AlreadyInitialized`: The router has already been initialized.
    *   `NotAuthorized`: The caller is not authorized to perform an admin operation.
    *   `AdapterMissing`: The requested adapter ID is not found in the registered list.
    *   `AdapterFailed`: An operation delegated to an adapter failed.
*   **`Key`**: An enum for storage keys:
    *   `Admin`: Stores the address of the router's administrator.
    *   `AdapterList`: Stores a `Vec<(i128, Address)>` mapping adapter IDs to their contract addresses.
*   **`LpPlan`**: A struct defining parameters for liquidity provision operations across different adapters:
    *   `adapter_id: i128`: The ID of the target adapter.
    *   `token_a: Address`: Address of the first token in the pair.
    *   `token_b: Address`: Address of the second token in the pair.
    *   `proportion: u32`: A simple weight or proportion for allocating liquidity to this plan.

### Traits and Client

*   **`RouterTrait`** (defined in `client.rs`): Outlines the public interface of the router contract.
    *   **Lifecycle Functions:**
        *   `initialize(e: Env, admin_addr: Address) -> Result<(), RouterError>`: Initializes the router, setting the admin address and an empty adapter list. Emits an `("router","init")` event.
        *   `upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), RouterError>`: Upgrades the router contract to a new WASM hash. Requires admin authorization.
    *   **Admin Operations:**
        *   `add_adapter(e: Env, id: i128, adapter: Address) -> Result<(), RouterError>`: Adds a new adapter or updates an existing one with the same ID. Requires admin authorization.
        *   `remove_adapter(e: Env, id: i128) -> Result<(), RouterError>`: Removes an adapter by its ID. Requires admin authorization.
        *   `admin(e: &Env) -> Address`: Returns the current admin address.
    *   **Swap Operations:**
        *   `swap_exact_in(...) -> Result<i128, RouterError>`: Routes a swap operation to the specified `adapter_id`. It calls the `try_swap_exact_in` method of the `AdapterClient`.
    *   **Liquidity Operations (Currently Stubs):**
        *   `provide_liquidity(...) -> Result<(), RouterError>`: Intended to manage liquidity provision across multiple adapters based on `LpPlan`s. Currently a `todo!()` stub.
        *   `redeem_liquidity(...) -> Result<(), RouterError>`: Intended to manage liquidity redemption. Currently a `todo!()` stub.
*   **`RouterClient`**: A Soroban contract client generated for `RouterTrait`.

### Internal Helper Functions

*   `adapter_addr(e: &Env, id: i128) -> Result<Address, RouterError>`: Retrieves the address of an adapter by its ID from the `AdapterList` in storage.

## Storage

*   `Key::Admin`: Stores the `Address` of the contract administrator.
*   `Key::AdapterList`: Stores a `Vec<(i128, Address)>` representing the list of registered adapters, where `i128` is the adapter ID and `Address` is its contract address.

## Events

*   `("router","init")`, `admin_addr: Address`: Emitted when the router is initialized.

## TODO

*   **Implement Liquidity Functions**: Fully implement `provide_liquidity` and `redeem_liquidity`. This will involve:
    *   Logic to interpret `LpPlan`s.
    *   Iterating through plans and interacting with the respective `AdapterClient`s (`add_liquidity`, `remove_liquidity`).
    *   Potentially handling token transfers to/from adapters and the beneficiary.
    *   Considering the minting/burning/management of a Hoops LP Token (HLPT) if that's part of the design for `provide_liquidity`.
*   **Advanced Routing Logic**: For `swap_exact_in` (and a potential `swap_exact_out`), consider implementing more advanced routing logic if the router is intended to find the best path/price across multiple registered adapters, rather than just taking an `adapter_id` as input.
*   **Error Handling**: Enhance error handling, especially around `AdapterFailed`, to potentially relay more specific errors from adapters if possible and useful.
*   **Gas Optimization**: Review storage access patterns and loops for gas efficiency, especially in admin operations and future liquidity functions.
*   **Security**: Add thorough checks and consider reentrancy guards if complex interactions with multiple external contracts (adapters) are implemented.
*   **Events**: Add more events for significant operations like adding/removing adapters, swaps, and liquidity operations.
