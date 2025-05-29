# Hoops Finance Contracts Workspace

This workspace contains all core smart contracts for the Hoops Finance protocol, including adapters for major AMMs, the router, account abstraction, and deployment utilities. Each contract is documented in its own directory. This file provides a high-level overview and next steps for development and testing.

## Structure

- **common/**: Shared types, traits, and error definitions.
- **adapter-interface/**: Canonical `AdapterTrait` and client for adapter contracts.
- **adapters/**: Protocol adapters for supported AMMs:
  - `aqua/`
  - `comet/`
  - `phoenix/`
  - `soroswap/`
- **router/**: The Hoops Router contract, orchestrating swaps and liquidity flows across adapters.
- **account/**: User account abstraction contract for secure, programmable interactions.
- **account_deployer/**: Utility for deploying new account contracts.
- **external_contracts/**: WASM and interfaces for external AMMs (for testing/integration).

## Development Status

- All core contracts and adapters are implemented and documented with individual `README.md` files.
- Each adapter implements the `AdapterTrait` for its respective protocol, with TODOs for protocol-specific improvements and error handling.
- The router and account contracts are ready for integration testing.

## Next Steps

1. **Integration & Unit Testing**
   - Write comprehensive tests for:
     - Swaps (all adapters, all edge cases)
     - Liquidity provision/removal
     - Account deployment and upgrade
     - Router admin and lifecycle
   - Test error conditions and revert scenarios.

2. **Adapter & Trait Refactoring**
   - Resolve duplication between `common` and `adapter-interface` traits and error types.
   - Standardize error handling and event emission across adapters.

3. **Feature Enhancements**
   - Implement missing liquidity functions in the router.
   - Add multi-signer and advanced authorization to accounts.
   - Improve adapter pool/LP token discovery and slippage protection.

4. **Documentation**
   - Expand contract-level and workspace-level documentation as features evolve.

## Getting Started

- See each contract's `README.md` for details on its interface, usage, and TODOs.
- Use `get-contracts.sh` to fetch or build required WASM artifacts for adapters and tests.
- Run tests with your preferred Soroban or Rust test runner.

---

**For contributors:**
- Please keep this file and all per-contract `README.md` files up to date as you make changes.
- Document all new features, breaking changes, and known issues.
