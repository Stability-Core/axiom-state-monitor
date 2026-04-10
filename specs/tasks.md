# Axiom State Monitor — Task Board

## [WAVE-TRIVIAL] Docs & Linting

- [ ] WAVE-001: Add `#![deny(missing_docs)]` to all contract crates
- [ ] WAVE-002: Write inline rustdoc for `StorageWatcher`, `RenewalEngine`, `AlertSystem`
- [ ] WAVE-003: Add `clippy` lint pass to CI (`cargo clippy -- -D warnings`)
- [ ] WAVE-004: Spell-check `specs/` markdown files via `cspell`
- [ ] WAVE-005: Add `rustfmt.toml` and enforce formatting in CI

## [WAVE-MEDIUM] Unit Tests for TTL Logic

- [ ] WAVE-006: Test `classify_state()` returns `Live` when ledger < `live_until_ledger_seq`
- [ ] WAVE-007: Test `classify_state()` returns `Archived` when ledger == `live_until_ledger_seq`
- [ ] WAVE-008: Test `get_ttl()` returns correct delta between current and expiry ledger
- [ ] WAVE-009: Test `ttl_warning` event is emitted when TTL <= threshold
- [ ] WAVE-010: Test `ttl_critical` event is emitted when TTL == 0
- [ ] WAVE-011: Test renewal is blocked for `Archived` state entries
- [ ] WAVE-012: Test renewal is blocked for `Dead` state entries
- [ ] WAVE-013: Test fee calculation matches expected `size * rate * ledgers` formula

## [WAVE-HIGH] Core Renewal Engine Implementation

- [ ] WAVE-014: Implement `extend_ttl(key, ledgers)` using `env.storage().persistent().extend_ttl()`
- [ ] WAVE-015: Implement `calc_rent_fee(entry_size, ledgers)` using ledger fee config
- [ ] WAVE-016: Integrate `admin.require_auth()` guard on all mutating entry points
- [ ] WAVE-017: Implement `restore_entry(key)` path for Archived state recovery
- [ ] WAVE-018: Wire `WatchedKeys` persistent vec — add/remove watched keys
- [ ] WAVE-019: Implement batch TTL check across all `WatchedKeys` in a single invocation
- [ ] WAVE-020: Emit structured Soroban Events with topics `["ttl_warning", key]`, `["ttl_critical", key]`
- [ ] WAVE-021: Deploy to Stellar Testnet and validate event indexing via Horizon API
