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

## Next steps

* Replace pseudocode in each `src/protocol_interface.rs` with the real function
  signatures once the external binaries are final.
* Fill `router::provide_liquidity` and `router::redeem_liquidity`.
* Write Tests
* Write Javascript Bindings

Created for the Hoops‑Fi project
