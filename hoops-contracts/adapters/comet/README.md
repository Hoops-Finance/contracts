# Comet Protocol Adapter Contract

The `comet-adapter` contract facilitates interaction with Comet Protocol's liquidity pools. It adheres to the `AdapterTrait` from `hoops-adapter-interface`, enabling the Hoops Finance `Router` to execute swaps and manage liquidity on Comet.

## Core Functionality

Implemented `AdapterTrait` functions:

*   `version() -> u32`: Returns `1`.
*   `initialize(e: Env, amm_id: i128, amm_addr: Address)`: Initializes the adapter.
    *   Checks if already initialized.
    *   Verifies `amm_id` is `1` (the `PROTOCOL_ID` for Comet in this adapter).
    *   Stores the `amm_addr` (Comet Pool address).
    *   Emits an `init` event.
*   `upgrade(e: Env, new_wasm_hash: BytesN<32>)`: Upgrades the contract WASM.
    *   Requires authorization from an admin address (fetched from `CoreConfig`).

### Swaps

*   `swap_exact_in(e: Env, amount_in: i128, min_out: i128, path: Vec<Address>, to: Address, deadline: u64) -> Result<i128, AdapterError>`:
    *   Performs a swap with a fixed input amount on a Comet pool.
    *   Currently supports only **single-hop swaps** (path must have 2 addresses).
    *   Calls `swap_exact_amount_in` on the Comet Pool contract.
    *   `max_price` is set to `i128::MAX`.
    *   Error handling for the Comet pool call seems incomplete.
*   `swap_exact_out(e: Env, amount_out: i128, max_in: i128, path: Vec<Address>, to: Address, deadline: u64) -> Result<i128, AdapterError>`:
    *   Performs a swap with a fixed output amount on a Comet pool.
    *   Currently supports only **single-hop swaps**.
    *   Calls `swap_exact_amount_out` on the Comet Pool contract.
    *   `max_price` is set to `i128::MAX`.
    *   Error handling for the Comet pool call seems incomplete.

### Liquidity Management

*   `add_liquidity(e: Env, token_a: Address, token_b: Address, amt_a: i128, amt_b: i128, to: Address, deadline: u64) -> Result<Address, AdapterError>`:
    *   Adds liquidity to a Comet pool.
    *   Calls `join_pool` on the Comet Pool contract.
    *   `max_amounts` are set to `amt_a` and `amt_b`.
    *   `pool_amount_out` (amount of LP tokens to mint) is currently simplified to `amt_a.min(amt_b)`. This needs a more accurate calculation based on the pool's state.
    *   Returns the Comet pool's address (`get_amm(&e)`) as the LP token address. This is correct as Comet pools are themselves the LP tokens.
    *   Error handling for the `join_pool` call seems incomplete.
*   `remove_liquidity(e: Env, lp_token: Address, lp_amount: i128, to: Address, deadline: u64) -> Result<(i128,i128), AdapterError>`:
    *   Removes liquidity from a Comet pool.
    *   Calls `exit_pool` on the Comet Pool contract.
    *   `min_amounts_out` is set to `[0, 0]`, accepting any amount of underlying tokens.
    *   **Critical**: Comet's `exit_pool` does not directly return the amounts of tokens withdrawn. The adapter currently returns a simplified `lp_amount / 2` for each token. This needs to be replaced with a mechanism to accurately determine or track the actual withdrawn amounts (e.g., by checking balances before and after, or if Comet emits an event with this info).
    *   Error handling for the `exit_pool` call seems incomplete.

## Protocol Interaction

*   The adapter interacts with a Comet pool contract, whose WASM is imported via `contractimport!` from `../../bytecodes/comet-pool.wasm`.
*   It uses `CometPoolClient` for these interactions.

## Storage

*   `AMM_ADDRESS_KEY`: Stores the `Address` of the Comet pool.
*   `INITIALIZED_KEY`: A boolean flag indicating if the adapter has been initialized.
*   `CoreConfig`: Stores admin address (presumably for `upgrade`).

## Events

*   `init(amm_addr: Address)`: Emitted during `initialize`.
*   (Swap and liquidity events are not explicitly shown in `lib.rs` but might be in `event.rs` - assumed to be similar to other adapters if present).

## Dependencies

*   `soroban-sdk`
*   `hoops-adapter-interface`
*   `hoops-common`

## TODOs & Potential Issues

*   **Multi-hop Swaps**: Extend swap functions to support multi-hop swaps if Comet protocol/pools allow for it directly or if it needs to be handled by chaining calls.
*   **Accurate LP Mint Calculation**: In `add_liquidity`, the `pool_amount_out` calculation needs to be accurate, likely involving querying the pool's reserves or current price.
*   **Accurate Withdrawal Amounts**: In `remove_liquidity`, implement a reliable way to determine the actual amounts of `token_a` and `token_b` withdrawn.
*   **Error Handling**: Complete the error handling for all calls to the Comet pool contract, mapping Comet-specific errors to `AdapterError` where possible.
*   **Admin for Upgrade**: Ensure `CoreConfig` and its admin field are properly set up during initialization or via a separate admin function if `upgrade` is to be functional.
*   **Slippage Protection**: The `max_price` in swaps and `min_amounts_out` in `remove_liquidity` are currently permissive. Allow users to specify slippage tolerance.
*   **Path to WASM**: Ensure the path to `comet-pool.wasm` is robust.
*   **Event Emission**: Ensure comprehensive events are emitted for all significant actions (swaps, adding/removing liquidity).
