# TODOs and Technical Notes for Hoops Router & Adapters

## Adapter Logic & Return Values
- [ ] Refactor Phoenix and Comet adapters’ `add_liquidity` to return correct `(amount_a, amount_b, lp_shares)` by querying balances before/after, since their pool contracts do not return these values directly.
- [x] Soroswap adapter’s `add_liquidity` now returns correct values as per router expectations.
- [ ] Ensure all adapters (Soroswap, Aqua, Phoenix, Comet) and their tests are consistent and robust for both swap and liquidity flows.
- [ ] Ensure `swap_exact_out` is tested for all adapters.

## Integration & Test Coverage
- [x] Created and registered per-adapter test files: `soroswap_adapter_tests.rs`, `aqua_adapter_tests.rs`, `phoenix_adapter_tests.rs`, `comet_adapter_tests.rs`.
- [x] Removed generic/smoke test files as requested.
- [x] Updated test setup to print and verify user LP balances for all protocols.
- [ ] Expand/verify tests for all adapters to cover edge cases and error handling.

## Rust/Testutils/Conditional Compilation
- [x] Investigated and explained Rust conditional compilation issues with `testutils.rs` (feature flags, `#[cfg(test)]`, etc.).
- [ ] Ensure all test utilities are only compiled for tests and do not bloat production WASM.

## Documentation & Tracking
- [ ] Keep this file updated with all key TODOs and technical decisions for future tracking.
- [ ] Document any protocol-specific quirks or workarounds in adapter/test logic.

## Miscellaneous
- [x] Provided guidance and code for copying .wasm files after build.
- [ ] Review and clean up any remaining test setup duplication or boilerplate.

---
**Legend:**
- [x] = Done
- [ ] = Pending

_Last updated: 2025-06-16_
