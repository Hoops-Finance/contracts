# Aqua Protocol Adapter Contract

The `aqua-adapter` contract serves as an intermediary to interact with the Aqua Protocol's liquidity pool router. It implements the `AdapterTrait` from `hoops-adapter-interface`, allowing the Hoops Finance `Router` to perform swaps and manage liquidity on Aqua.

## Core Functionality

Implemented `AdapterTrait` functions:

*   `version() -> u32`: Returns `1`.
*   `initialize(e: Env, amm_id: i128, amm_addr: Address)`: Initializes the adapter.
    *   Checks if already initialized.
    *   Verifies `amm_id` is `0` (the `PROTOCOL_ID` for Aqua in this adapter).
    *   Stores the `amm_addr` (Aqua Router address).
    *   Emits an `init` event.
*   `upgrade(_e: Env, _hash: BytesN<32>)`: Currently a stub, always returns `AdapterError::ExternalFailure`.

### Swaps

*   `swap_exact_in(e: Env, amt_in: i128, min_out: i128, path: Vec<Address>, to: Address, deadline: u64) -> Result<i128, AdapterError>`:
    *   Performs a swap with a fixed input amount on Aqua.
    *   Converts the generic `path` into Aqua's `swaps_chain` format.
        *   Note: The `pool_index` for each hop in the chain is currently defaulted to `BytesN::from_array(&e, &[0; 32])`. This might need to be more dynamic based on actual Aqua pool configurations.
    *   Calls the `swap_chained` function on the Aqua Router.
    *   Emits a `swap` event.
*   `swap_exact_out(e: Env, out: i128, max_in: i128, path: Vec<Address>, to: Address, deadline: u64) -> Result<i128, AdapterError>`:
    *   Performs a swap with a fixed output amount on Aqua.
    *   Similar to `swap_exact_in`, converts the `path` to `swaps_chain` with a default `pool_index`.
    *   Calls the `swap_chained_strict_receive` function on the Aqua Router.
    *   Emits a `swap` event.

### Liquidity Management

*   `add_liquidity(e: Env, a: Address, b: Address, amt_a: i128, amt_b: i128, to: Address, deadline: u64) -> Result<Address, AdapterError>`:
    *   Adds liquidity to an Aqua pool.
    *   Calls the `deposit` function on the Aqua Router.
    *   `pool_index` is defaulted.
    *   `min_shares` is defaulted to `1u128`.
    *   **Critical**: The function currently returns `to.clone()` as the LP token address, which is a placeholder. It needs to correctly determine and return the actual LP token/pool address from Aqua.
    *   Error handling for the `router.deposit` call seems to be commented out or incomplete.
*   `remove_liquidity(e: Env, lp: Address, lp_amt: i128, to: Address, deadline: u64) -> Result<(i128,i128), AdapterError>`:
    *   Removes liquidity from an Aqua pool.
    *   Calls the `withdraw` function on the Aqua Router.
    *   **Critical**: The `tokens` vector (representing the underlying tokens of the LP) is initialized as empty. This needs to be fetched or determined correctly for the specific `lp` address.
    *   `pool_index` is defaulted.
    *   `min_amounts` is defaulted to an empty vector.
    *   Error handling for the `router.withdraw` call seems tobe commented out or incomplete.

## Protocol Interaction

*   The adapter interacts with the Aqua router contract whose WASM is imported via `contractimport!` from `../../target/wasm32v1-none/release/aqua_liquidity_pool_router_contract.wasm`.
*   It uses `AquaRouterClient` for these interactions.

## Storage

*   `AMM_ADDRESS_KEY`: Stores the `Address` of the Aqua router.
*   `INITIALIZED_KEY`: A boolean flag indicating if the adapter has been initialized.

## Events

*   `init(amm_addr: Address)`: Emitted during `initialize`.
*   `swap(amt_in: i128, amt_out: i128, path: Vec<Address>, to: Address)`: Emitted after successful swaps.

## Dependencies

*   `soroban-sdk`
*   `hoops-common`
*   `hoops-adapter-interface`

## TODOs & Potential Issues

*   **Upgrade Functionality**: Implement the `upgrade` function.
*   **Dynamic Pool Index**: The `pool_index` in swap and liquidity functions is currently hardcoded/defaulted. This needs to be dynamically determined based on the token pairs or specific Aqua pool identifiers.
*   **Correct LP Token Address**: In `add_liquidity`, the actual LP token address from Aqua must be returned, not the `to` address.
*   **Underlying Token Discovery**: In `remove_liquidity`, the `tokens` vector (underlying assets of the LP token) must be correctly populated.
*   **Error Handling**: The commented-out `.map_err(|_| AdapterError::ExternalFailure)` suggests incomplete error handling for calls to the Aqua router. This needs to be properly implemented to translate Aqua-specific errors into `AdapterError` variants if possible, or at least reliably signal `ExternalFailure`.
*   **Minimum Amounts/Shares**: The `min_shares` in `add_liquidity` and `min_amounts` in `remove_liquidity` are currently hardcoded or empty. These should ideally be parameters or derived more intelligently to protect against slippage.
*   **Path to WASM**: Ensure the path to `aqua_liquidity_pool_router_contract.wasm` is robust for different build and deployment environments.
*   **Aqua Protocol Specifics**: Thoroughly review Aqua Protocol documentation to ensure all parameters (like `pool_index`) and interaction patterns are correctly implemented.
*   **Gas & Bumping**: Review `bump()` calls to ensure state longevity, especially around external calls.
