# Hoops Soroban Contracts Template

Modular Soroban smart‑contract suite that lets a user deposit one asset (e.g. USDC) and spread it across several AMMs for swaps or liquidity‑provision—currently **Aqua (0)**, **Comet (1)**, **Soroswap (2)**, and **Phoenix (3)**
---

## Directory layout

```sh

contracts/
├─ common/                 shared admin/events/error helpers
├─ adapter-interface/      trait every AMM adapter must follow
├─ adapters/
│   ├─ aqua/               adapter‑id 0
│   ├─ comet/              adapter‑id 1
│   ├─ soroswap/           adapter‑id 2
│   └─ phoenix/            adapter‑id 3
├─ router/                 swap & liquidity dispatcher
├─ account/                user‑owned contract that holds balances
└─ account\_deployer/       deploys Account atomically

```

### External AMM binaries

We need to add all the compiled contracts to the adapters/{protocol}/{protocol}-contracts locations.

---

## Adapter IDs

| ID | Protocol | Crate          |
|---:|----------|---------------|
| 0  | Aqua     | `aqua-adapter`|
| 1  | Comet    | `comet-adapter`|
| 2  | Soroswap | `soroswap-adapter`|
| 3  | Phoenix  | `phoenix-adapter`|

These `i128` values are what the router and front‑end use to choose an adapter.

---

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --workspace --target wasm32-unknown-unknown --release
````

Optimised `.wasm` files are produced under
`target/wasm32-unknown-unknown/release/`.

---

## Contract flow

1. **Account** – smart account owned by the user.
   *Receives USDC, approves the Router, and calls deposit / withdraw.*

2. **Router** – takes USDC, splits it per user‑supplied plan, calls the correct
   adapter(s) to swap or add/remove liquidity.

3. **Adapters** – one per AMM.
   *Translate the Router’s generic request into the AMM’s concrete contract
   calls (swap, add LP, remove LP).*

---

## Authentication

The Account contract supports two authentication modes:

1. **External wallet (Ed25519)** — Traditional Stellar wallets (Freighter) using `require_auth()` on the owner address.
2. **Passkey (secp256r1)** — Non-custodial biometric auth via WebAuthn. Implements `CustomAccountInterface` so all `require_auth()` calls route through `__check_auth()` for on-chain P-256 signature verification.

See [`account/README.md`](hoops-contracts/account/README.md) for full passkey documentation.

---

## Testnet Deployment

All contracts are deployed and initialized on Stellar Testnet (Feb 2026):

| Contract | Address |
|----------|---------|
| Router | `CCXJKZWHXCN4JYPHOWJWHTIE7NTTPK6JOZNMYNRKX3IEWD4OW6O7V6AE` |
| AccountDeployer | `CDBKXYD3HHYUSHPRVTGDQRVAHRVDRLIUWER2KZRIAHYLPSOARXEX54Z6` |
| Aqua Adapter | `CBNYDXFV5GTNRXFPY5KZ6W3BP5NLYDAMHY4ZZYZGHQF4JTMDZWQDOPNZ` |
| Comet Adapter | `CBBKYGYJG2ONO22EAEP6N532UTDNQQELJVCGPBX22NFPP67YBW4UWQLX` |
| Phoenix Adapter | `CBQAZAMW3L4V45SXFEXYJMBQSBOIPLOUXCCVONIV5DZ7BIHCVXECAURN` |
| Soroswap Adapter | `CABSGGC6Y6ERN6WIPOP22IOKMZZLH2XES7YCAL6VWRCNFFO3MCS7N7IL` |

**Account WASM hash (passkey):** `adecd3f562a43a1d15860fc1c21c9c29632be9267926c2289dcdc51de482460e`

---

Created for the Hoops‑Fi project
