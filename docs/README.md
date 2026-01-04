# Hoops Contracts Documentation

## Overview / Purpose

Hoops Contracts is a modular Soroban smart contract suite that enables users to deposit a single asset (e.g., USDC) and distribute it across multiple Automated Market Makers (AMMs) for swaps or liquidity provision. The system currently supports four AMM protocols: **Aqua (0)**, **Comet (1)**, **Soroswap (2)**, and **Phoenix (3)**.

The project provides a unified interface for interacting with multiple AMMs through a router contract that intelligently routes transactions to the appropriate adapter based on user-specified plans.

## Architecture Summary

### Key Components

- **Router**: Central dispatcher contract that receives user deposits, splits them according to user-supplied plans, and calls the appropriate adapter(s) for swaps or liquidity operations
- **Adapters**: Protocol-specific adapters that translate the router's generic requests into concrete AMM contract calls
  - `aqua/`: Adapter for Aqua protocol (adapter-id 0)
  - `comet/`: Adapter for Comet protocol (adapter-id 1)
  - `soroswap/`: Adapter for Soroswap protocol (adapter-id 2)
  - `phoenix/`: Adapter for Phoenix protocol (adapter-id 3)
- **Account**: User-owned smart contract account that holds balances and manages approvals
- **Account Deployer**: Utility contract for deploying new account contracts atomically
- **Common**: Shared types, traits, error definitions, and admin/event helpers
- **Adapter Interface**: Canonical `AdapterTrait` that all adapters must implement

### Data Flow

1. User deposits assets into their **Account** contract
2. User approves the **Router** contract to spend their assets
3. User calls deposit/withdraw on the Router with a distribution plan
4. **Router** splits the assets according to the plan and calls the appropriate **Adapter(s)**
5. **Adapters** translate requests into protocol-specific contract calls (swap, add LP, remove LP)
6. Results are aggregated and returned to the user

### Dependencies

- **Soroban SDK**: Version 22.0.8
- **External AMM Contracts**: Compiled WASM binaries for Aqua, Comet, Soroswap, and Phoenix protocols
- **Rust**: Version 1.87.0 or higher
- **Build Target**: `wasm32-unknown-unknown` for contract compilation

## Setup Instructions

### Prerequisites

- Rust toolchain (1.87.0+)
- `rustup` with `wasm32-unknown-unknown` target
- Soroban CLI tools
- Access to Stellar network (testnet or mainnet)

### Installation

```bash
# Install Rust target for WASM compilation
rustup target add wasm32-unknown-unknown

# Clone the repository (if not already done)
# git clone <repository-url>
# cd hoops_contracts
```

### Build Commands

```bash
# Build all contracts
cargo build --workspace --target wasm32-unknown-unknown --release

# Or use the Makefile
make build

# Build with diagnostic logs
stellar contract build --out-dir bytecodes --profile release-with-logs

# Build directly to bytecodes folder
stellar contract build --out-dir bytecodes
```

### Running Tests

```bash
# Run all tests (single-threaded recommended)
cargo test --test-threads=1

# Run specific test with logs
cargo test test_soroswap_adapter_swap_exact_in -- --nocapture --test-threads=1

# Run tests for specific contract
cd router
cargo test -- --nocapture --test-threads=1
```

### Post-Build Steps

After building contracts, copy WASM files to the bytecodes folder:

```bash
find ./target/wasm32v1-none/release -type f -name '*.wasm' -exec cp {} ./bytecodes/ \;
```

## Deployment Instructions

### Cloud Deployment

_Placeholder for cloud deployment instructions_

- Network configuration (testnet/mainnet)
- RPC endpoint setup
- Contract deployment sequence
- Admin key management

### Docker Deployment

_Placeholder for Docker deployment instructions_

- Dockerfile configuration
- Container orchestration
- Environment variable management
- Volume mounts for bytecodes

### Deployment Checklist

- [ ] Configure network endpoints
- [ ] Set up admin keys securely
- [ ] Deploy external AMM contracts (if needed)
- [ ] Deploy adapter contracts
- [ ] Deploy router contract
- [ ] Deploy account deployer
- [ ] Verify contract addresses
- [ ] Initialize contracts with proper parameters

## Known Issues / Blockers

- [ ] Missing liquidity functions in router (`provide_liquidity` and `redeem_liquidity` need implementation)
- [ ] Some adapters need refactoring to return correct values for `add_liquidity` (Phoenix and Comet)
- [ ] Test coverage needs expansion for edge cases and error handling
- [ ] Duplication between `common` and `adapter-interface` traits needs resolution
- [ ] External AMM binaries need to be added to `adapters/{protocol}/{protocol}-contracts` locations
- [ ] JavaScript bindings need to be written
- [ ] Multi-signer and advanced authorization features for accounts pending

## Owner & Status

**Project**: Hoops Finance Contracts  
**Status**: Active Development  
**Repository**: https://github.com/hoops-fi/contracts  
**License**: Apache-2.0

---

_This documentation is a work in progress. Please refer to individual contract README files for detailed implementation notes._

