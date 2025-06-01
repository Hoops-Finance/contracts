# Phoenix Protocol Adapter Contract

The `phoenix-adapter` contract enables the Hoops Finance Router to interact with Phoenix AMM pools for swaps and liquidity management. It implements the `AdapterTrait` from `hoops-adapter-interface` and acts as a bridge to the Phoenix pool contract.

## Core Functionality

### Lifecycle
- **initialize**: Sets up the adapter for Phoenix (PROTOCOL_ID = 2), storing the pool address and marking the adapter as initialized.
- **upgrade**: Currently a stub, always returns `AdapterError::ExternalFailure`.
- **version**: Returns the adapter version (1).

### Swaps
- **swap_exact_in**: Swaps a fixed input amount for as much output as possible, using the Phoenix pool's `swap` method. Only single-hop swaps are supported.
- **swap_exact_out**: Swaps as little input as possible to receive a fixed output amount, using the pool's `simulate_reverse_swap` to determine the required input, then calling `swap`.

### Liquidity Management
- **add_liquidity**: Adds liquidity to a Phoenix pool via the `provide_liquidity` method. Both token amounts must be > 0. Returns the `to` address as a placeholder for the LP token (should be replaced with actual logic if Phoenix supports LP tokens).
- **remove_liquidity**: Removes liquidity from a Phoenix pool via `withdraw_liquidity`. Returns the withdrawn amounts for each token.

## Protocol Interaction
- Uses `PhoenixPoolClient` (imported from WASM) to interact with Phoenix pool contracts.
- The pool WASM is imported from the `bytecodes/phoenix_pool.wasm` file.

## Storage
- Stores the pool address and initialization state.

## Events
- Swap and liquidity events are referenced but not fully implemented in the provided code (see `event.rs`).

## Dependencies
- `soroban-sdk`
- `hoops-adapter-interface`
- `hoops-common`

## TODOs & Next Steps
- **LP Token Address**: If Phoenix supports LP tokens, replace the placeholder in `add_liquidity` with logic to return the actual LP token address.
- **Error Handling**: Implement proper error handling for all Phoenix pool calls, mapping errors to `AdapterError`.
- **Event Emission**: Ensure all swap and liquidity actions emit appropriate events for off-chain tracking.
- **Parameterization**: Allow for slippage and minimum/maximum amounts to be set by the caller, not hardcoded.
- **Testing**: Write integration tests for all swap and liquidity flows, including edge cases and error conditions.
- **Upgrade Functionality**: Implement a real upgrade path if needed.
- **WASM Path Robustness**: Make sure the WASM import path is reliable for all build/deploy environments.
