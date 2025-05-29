# Soroswap Protocol Adapter Contract

The `soroswap-adapter` contract enables the Hoops Finance Router to interact with Soroswap's AMM protocol for swaps and liquidity management. It implements the `AdapterTrait` from `hoops-adapter-interface` and acts as a bridge to the Soroswap router and pair contracts.

## Core Functionality

### Lifecycle
- **initialize**: Sets up the adapter for Soroswap (PROTOCOL_ID = 3), storing the router address and marking the adapter as initialized.
- **upgrade**: Allows contract upgrade by an admin (from `CoreConfig`).
- **version**: Returns the adapter version (1).

### Swaps
- **swap_exact_in**: Swaps a fixed input amount for as much output as possible, using the Soroswap router's `swap_exact_tokens_for_tokens`. Returns the output amount (last in the returned vector).
- **swap_exact_out**: Swaps as little input as possible to receive a fixed output amount, using the router's `swap_tokens_for_exact_tokens`. Returns the input amount (first in the returned vector).

### Liquidity Management
- **add_liquidity**: Adds liquidity to a Soroswap pair via the router. Returns the LP token address (currently a placeholder: `to`). In a real implementation, this should query the factory for the actual pair address.
- **remove_liquidity**: Removes liquidity from a Soroswap pair. Determines the underlying tokens using the pair contract, then calls the router's `remove_liquidity`. Returns the withdrawn amounts for each token.

## Protocol Interaction
- Uses `SoroswapRouterClient` and `SoroswapPairClient` (imported from WASM) to interact with Soroswap contracts.
- The router and pair WASMs are imported from the `target/wasm32v1-none/release/` directory.

## Storage
- Stores the router address and initialization state.
- Uses `CoreConfig` for admin (upgrade authorization).

## Events
- Swap and liquidity events are referenced but not fully implemented in the provided code (see `event.rs`).

## Dependencies
- `soroban-sdk`
- `hoops-adapter-interface`
- `hoops-common`

## TODOs & Next Steps
- **LP Token Address**: Replace the placeholder in `add_liquidity` with logic to query the actual pair address from the Soroswap factory.
- **Error Handling**: Implement proper error handling for all Soroswap router/pair calls, mapping errors to `AdapterError`.
- **Event Emission**: Ensure all swap and liquidity actions emit appropriate events for off-chain tracking.
- **Parameterization**: Allow for slippage and minimum/maximum amounts to be set by the caller, not hardcoded.
- **Testing**: Write integration tests for all swap and liquidity flows, including edge cases and error conditions.
- **Documentation**: Expand on the purpose and usage of the `init` function (currently unclear in the code).
- **Upgrade Robustness**: Ensure `CoreConfig` is always set up correctly for secure upgrades.
- **WASM Path Robustness**: Make sure the WASM import paths are reliable for all build/deploy environments.
