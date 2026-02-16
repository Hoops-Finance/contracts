# Hoops Account Contract

The `hoops-account` contract acts as a user-specific smart account designed to streamline interactions with the Hoops Finance `Router`. It enables users to manage their liquidity provision, redemption, and swap activities within the Hoops ecosystem.

The contract supports two authentication modes:
- **External wallet (Ed25519)**: Traditional Stellar wallets (e.g., Freighter) using `require_auth()` on the owner address.
- **Passkey (secp256r1)**: Non-custodial biometric authentication via WebAuthn. Once a passkey public key is set, all `require_auth()` calls route through `__check_auth()` for on-chain P-256 signature verification.

## Core Functionality

### Lifecycle Management

- `initialize(owner, router)` — Sets the owner and router addresses. Can only be called once.
- `initialize_with_passkey(owner, router, passkey_pubkey)` — Initializes the account with a passkey as primary signer. Stores the 65-byte uncompressed secp256r1 public key alongside owner and router.
- `upgrade(wasm)` — Allows the owner to upgrade the contract's WASM code.

### Passkey Authentication

- `set_passkey_pubkey(pubkey: BytesN<65>)` — Store or replace the passkey public key. If a passkey is already set, the caller must authenticate via the existing passkey (routed through `__check_auth`).
- `get_passkey_pubkey() -> Option<BytesN<65>>` — Returns the stored passkey public key, or None if not set.
- `__check_auth(signature_payload, signature, auth_contexts)` — Implements `CustomAccountInterface`. Verifies secp256r1 signatures using the WebAuthn verification pipeline:
  1. Load stored passkey public key
  2. Concatenate `authenticator_data || SHA-256(client_data_json)`
  3. Verify secp256r1 signature on `SHA-256(concatenated_data)`
  4. Parse `client_data_json` to extract the `challenge` field
  5. Verify challenge matches `base64url(signature_payload)`

### Token Passthrough

- `transfer(token, to, amount)` — Transfer any token held by this account to a specified address. Requires owner auth.

### Liquidity Operations

- `deposit(usdc, amount, lp_plans, deadline)` — Transfers tokens from the Account to the Router for each LP plan, then calls `provide_liquidity` on the Router. The `lp_plans` vector specifies how funds are allocated across pools.
- `redeem(lp_token, lp_amount, usdc, deadline)` — Approves the Router to pull LP tokens, calls `redeem_liquidity`, then sweeps received USDC to the owner.

### Swap

- `swap(token_in, token_out, amount, best_hop, deadline)` — Transfers `token_in` to the Router and executes a swap via the best hop address.

### View Functions

- `owner() -> Address` — Returns the account owner.
- `router() -> Address` — Returns the Hoops Router contract address.

## Types

### Secp256r1Signature

```rust
pub struct Secp256r1Signature {
    pub authenticator_data: Bytes,
    pub client_data_json: Bytes,
    pub signature: BytesN<64>,
}
```

WebAuthn signature payload for passkey authentication. Produced by the browser's WebAuthn API during biometric authentication.

### LpPlan

```rust
pub struct LpPlan {
    pub token_a: Address,
    pub token_b: Address,
    pub amount_a: i128,
    pub amount_b: i128,
    pub adapter_id: i128,
}
```

Specifies how funds should be allocated to a specific DEX adapter.

## Error Handling

| Code | Variant | Description |
|------|---------|-------------|
| 1 | `AlreadyInitialized` | `initialize` or `initialize_with_passkey` called more than once |
| 2 | `NotAuthorized` | Caller lacks authorization |
| 3 | `PasskeyNotSet` | `__check_auth` called but no passkey public key is stored |
| 4 | `ClientDataJsonChallengeIncorrect` | WebAuthn challenge doesn't match expected signature payload |
| 5 | `JsonParseError` | Failed to parse `client_data_json` from the WebAuthn response |

## Events

The contract emits `TokenEvent { token: Address, amount: i128 }` for:
- `("acct", "xfer")` — On successful `transfer`
- `("acct", "dep")` — On successful `deposit`
- `("acct", "wd")` — On successful `redeem` (logs USDC swept to owner)
- `("acct", "swap")` — On successful `swap`

## Storage Keys

| Key | Type | Description |
|-----|------|-------------|
| `Owner` | `Address` | Account owner (EOA or deployer) |
| `Router` | `Address` | Hoops Router contract address |
| `PasskeyPubkey` | `BytesN<65>` | Uncompressed secp256r1 public key (0x04 prefix + x + y) |

## Dependencies

- `soroban-sdk` — Soroban smart contract SDK
- `hoops-common` — Shared types
- `hoops-router` — Router client (WASM imported via `contractimport!`)
- `serde` — Serialization for JSON parsing
- `serde-json-core` — No-std JSON parser for `client_data_json`

## Testing

```bash
# Run all account tests
cargo test --package hoops-account -- --test-threads=1

# Build WASM
stellar contract build --out-dir bytecodes --package hoops-account
```

## Testnet Deployment

- **Hoops Account WASM hash**: `adecd3f562a43a1d15860fc1c21c9c29632be9267926c2289dcdc51de482460e`
- **Deployer**: `GDWY4J3KME3QD4FOL5SMHCUAXUM56SAKXFDPHBLUU4WYATUPUVZKDCLB`
