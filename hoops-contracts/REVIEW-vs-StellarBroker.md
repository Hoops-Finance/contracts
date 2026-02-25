# Hoops Finance vs StellarBroker: Gap Analysis

_Review date: 2025-02-25_

## What StellarBroker Is

[StellarBroker](https://stellar.broker/) is a multi-source liquidity swap router for Stellar. Its architecture is fundamentally different from Hoops — it's primarily an **off-chain server** with a thin on-chain smart contract layer.

### StellarBroker Architecture

| Component | Description |
|---|---|
| **Router Server** | Backend service with REST API + WebSocket gateways |
| **Matcher Engine** | In-memory graph replicating Stellar Core's pathfinding, with **trade splitting** across parallel routes |
| **Ledger State Reader** | Reads blockchain state directly from Stellar Core node, feeds in-memory DEX Graph + Soroban Graph |
| **Graph Loader** | Protocol-specific providers that parse orderbook & AMM state |
| **Shard Dispatcher** | Per-CPU-core copies of the state graph for max throughput |
| **Transaction Submitter** | Simulates & submits split transactions in parallel within the same ledger via Channel Pool |
| **Router Contract** | Thin Soroban contract facilitating multi-hop swaps prepared by the server |
| **Client Library** | `@stellar-broker/client` npm package with WebSocket sessions, quote streaming, delegated signing |
| **UI** | Web frontend at stellar.broker |
| **MongoDB** | Swap history storage |

### StellarBroker Router Contract Structure

```
router-contract/src/
├── lib.rs              # Main entry: swap(), init(), enable_protocol(), update_contract(), withdraw()
├── auth.rs             # Authorization helpers (require_admin, add_transfer_auth, add_approve_auth)
├── storage.rs          # Persistent state (admin, fee token, protocol flags)
├── adapters/
│   ├── adapter.rs      # Common AdapterTrait
│   ├── soroswap.rs     # SoroSwap integration
│   ├── aqua_constant.rs # Aquarius constant product pools
│   ├── aqua_stable.rs  # Aquarius stable swap pools
│   ├── comet.rs        # Comet integration
│   └── phoenix.rs      # Phoenix integration
└── types/
    ├── protocol.rs     # Protocol enum (AquaConstant=0, AquaStable=1, Soroswap=2, Comet=3, Phoenix=4)
    ├── route.rs        # Route struct (path, amount, min, estimated)
    ├── step.rs         # PathStep struct (protocol, asset, pool, selling/buying index)
    ├── swapinfo.rs     # LPSwap struct
    └── error.rs        # BrokerError enum
```

Key difference: StellarBroker's contract receives **pre-computed routes** from the off-chain server. The contract just executes them. All intelligence lives server-side.

### StellarBroker Liquidity Sources (7 total)

| Source | Type |
|---|---|
| Stellar Classic DEX (SDEX orderbook) | Classic |
| Stellar Classic AMMs | Classic |
| Aquarius (constant product pools) | Soroban |
| Aquarius (stable swap pools) | Soroban |
| SoroSwap | Soroban |
| Comet | Soroban |
| Phoenix | Soroban |

---

## What Hoops Finance Has

Hoops architecture is **on-chain first** — the smart contracts do routing, quoting, and execution.

| Component | Description | Status |
|---|---|---|
| **Router Contract** | On-chain orchestrator for swaps + liquidity across adapters | Done |
| **Adapter Interface** | Canonical `AdapterTrait` (swap, quote, liquidity) | Done |
| **Soroswap Adapter** (ID: 3) | Full implementation (swaps, liquidity, quotes) | Complete |
| **Aqua Adapter** (ID: 0) | Swaps + liquidity, but LP token tracking & pool index hardcoded | Partial |
| **Phoenix Adapter** (ID: 2) | Single-hop only, upgrade stub, LP placeholder | Partial |
| **Comet Adapter** (ID: 1) | Single-hop, simplified LP calcs, no slippage protection | Partial |
| **Account Contract** | Smart account w/ Ed25519 + WebAuthn/passkey auth | Complete |
| **Account Deployer** | Factory for deploying user accounts | Complete |
| **Pool Discovery** | Only Soroswap implemented; Aqua/Phoenix/Comet are stubs | Partial |
| **Client Library** | None | Missing |
| **Off-chain Router/Matcher** | None | Missing |
| **UI** | None (in this repo) | Missing |

---

## Key Architectural Differences

### 1. On-chain vs Off-chain Routing

| | Hoops (on-chain) | StellarBroker (off-chain) |
|---|---|---|
| **Trust model** | Fully trustless, all logic verifiable on-chain | Non-custodial but server-mediated |
| **Complexity ceiling** | Limited by Soroban execution budget (gas/CPU) | Unlimited computation for pathfinding |
| **Trade splitting** | Not implemented — picks single best route | Splits across multiple routes in parallel |
| **Latency** | Single tx per swap | Multi-tx, parallel submission in same ledger |
| **Multi-hop** | Not implemented (single-hop only) | Full multi-hop across any DEX |
| **Quotes/sec** | On-chain calls per adapter | Tens of thousands via in-memory graph |

### 2. Liquidity Coverage

| Source | Hoops | StellarBroker |
|---|---|---|
| Soroswap | Yes | Yes |
| Aquarius (Aqua) | Yes (partial) | Yes (both pool types) |
| Phoenix | Yes (partial) | Yes |
| Comet | Yes (partial) | Yes |
| Stellar Classic DEX (orderbooks) | **No** | **Yes** |
| Stellar Classic AMMs | **No** | **Yes** |

### 3. What StellarBroker Has That Hoops Doesn't

| Feature | Notes |
|---|---|
| **Trade splitting** | Splits large orders across multiple routes for better execution |
| **Parallel execution** | Submits split txs simultaneously via channel accounts |
| **Multi-hop routing** | Arbitrary-length swap paths (A→B→C→D) |
| **Classic DEX + Classic AMM** | Taps native Stellar orderbooks |
| **Real-time state graph** | In-memory replica of all DEX/AMM state, updated per ledger |
| **WebSocket streaming** | Live quote updates via persistent connections |
| **Client SDK** | `@stellar-broker/client` npm package |
| **Delegated signing** | Mediator accounts for multisig/delayed signing |
| **Formal verification** | Audited by Runtime Verification using Komet |
| **Production UI** | Live at stellar.broker |
| **Fee model** | Variable fees based on estimated vs actual execution + percentage |
| **Protocol enable/disable** | Admin can toggle protocols on/off |
| **Stable swap pool support** | Aquarius stable pools (optimized for correlated assets) |

### 4. What Hoops Has That StellarBroker Doesn't

| Feature | Notes |
|---|---|
| **Smart account abstraction** | User-owned accounts with passkey (WebAuthn/secp256r1) auth |
| **On-chain liquidity provision** | Router handles `provide_liquidity` and `redeem_liquidity` across adapters |
| **On-chain quote aggregation** | `get_all_quotes` / `get_best_quote` — fully trustless |
| **DeFindex pattern** | Shallow auth chain design for composability |
| **Fully trustless routing** | No server dependency — everything verifiable on-chain |

---

## Distance Assessment

### What's Working Well (~65% of on-chain foundation)

- Core adapter pattern architecture is clean and extensible
- Soroswap adapter is production-quality
- Account contract with passkey auth is a genuine differentiator
- Testnet deployment is live with all contracts initialized
- Test infrastructure is comprehensive (per-adapter test suites)

### Gaps to Reach StellarBroker Parity

#### Tier 1 — Critical (required to be competitive)

1. **Finish 3 partial adapters** — Aqua (LP tracking, pool index), Phoenix (multi-hop, upgrade), Comet (LP calcs, slippage). Each ~60-80% done.

2. **Multi-hop routing** — Current router does single-hop only. StellarBroker finds arbitrary-length paths. Essential for tokens without direct pools.

3. **Pool discovery for all protocols** — Only Soroswap's `discover_*_pools` implemented. Other 3 are empty stubs.

4. **Trade splitting** — Single-route execution gets worse prices at volume. StellarBroker's core advantage.

#### Tier 2 — Important for production

5. **Classic DEX / Classic AMM integration** — Missing two of Stellar's deepest liquidity sources. Likely needs off-chain component.

6. **Off-chain quote optimizer** — Hybrid approach: off-chain pathfinding + on-chain execution.

7. **Client SDK** — JS/TS library wrapping contract calls, quote polling, tx building.

8. **Security audit** — StellarBroker was formally verified by Runtime Verification.

9. **Fee model** — StellarBroker implements variable fees. Hoops has no fee structure.

10. **Aqua stable swap pools** — StellarBroker distinguishes between constant product and stable pools.

#### Tier 3 — Nice-to-have

11. **Parallel tx submission** — Channel accounts for same-ledger execution.
12. **WebSocket quote streaming** — Real-time updates.
13. **Slippage protection** — Currently missing or hardcoded to 0 in several adapters.
14. **Protocol enable/disable** — Admin toggle for protocols (useful for emergencies).

---

## Progress Summary

| Dimension | Hoops Progress | Gap |
|---|---|---|
| On-chain contracts (adapters, router) | ~65% | Finish adapters, add multi-hop, pool discovery |
| Swap routing intelligence | ~20% | Multi-hop, trade splitting, off-chain optimizer |
| Liquidity coverage | ~50% | Missing Classic DEX + Classic AMMs |
| Client/SDK | 0% | Need JS client library |
| UI | 0% | Need frontend |
| Production readiness (audit, monitoring) | ~10% | Audit, monitoring, error recovery |
| **Unique advantages** | Account abstraction + passkey auth, on-chain LP management | N/A |

---

## Recommended Strategy

The most pragmatic path forward is a **hybrid approach**:

1. **Keep on-chain router** for execution and trustless verification
2. **Build off-chain service** for pathfinding and quote optimization that submits through existing contracts
3. **Lean into passkey-based smart accounts** as a genuine UX differentiator StellarBroker lacks
4. **Add Classic DEX/AMM access** through the off-chain layer (path payments can access classic liquidity without Soroban contracts)
5. **Prioritize finishing adapters** as the most impactful near-term work

The on-chain liquidity management (deposit/redeem across protocols) is a feature StellarBroker doesn't offer — this could be a strong differentiator for users who want yield/LP exposure, not just swaps.
