# Axiom State Monitor — Requirements (EARS Notation)

## 1. Ubiquitous Requirements

REQ-001: The system SHALL be implemented in safe Rust 1.75+ with `no_std` compatibility.
REQ-002: The system SHALL use `soroban-sdk` for all on-chain interactions.
REQ-003: The system SHALL compile to a WASM target suitable for Stellar Soroban deployment.

## 2. Event-Driven Requirements

REQ-004: WHEN a Persistent storage entry's `live_until_ledger_seq` falls within a configurable threshold, the system SHALL emit a Soroban Event of type `ttl_warning`.
REQ-005: WHEN a storage entry's TTL reaches zero, the system SHALL emit a Soroban Event of type `ttl_critical`.
REQ-006: WHEN the Renewal Engine is invoked, the system SHALL calculate the rent fee required to extend TTL by a caller-specified number of ledgers.

## 3. State-Driven Requirements

REQ-007: WHILE a storage entry is in the `Live` state, the Storage Watcher SHALL be able to query its `live_until_ledger_seq`.
REQ-008: WHILE a storage entry is in the `Archived` state, the system SHALL NOT attempt a TTL renewal and SHALL emit an `archived_entry` event.

## 4. Optional Requirements

REQ-009: WHERE the operator configures a minimum TTL buffer, the Renewal Engine SHOULD automatically extend TTL without manual invocation.
REQ-010: WHERE benchmarking scripts are available, the system SHOULD report gas/fee estimates before committing renewal transactions.

## 5. Unwanted Behaviour

REQ-011: The system SHALL NOT extend TTL for entries already in the `Dead` (permanently deleted) state.
REQ-012: The system SHALL NOT allow unsigned or unauthenticated renewal calls.

## 6. Glossary

| Term | Definition |
|------|-----------|
| TTL | Time To Live — ledgers remaining before a storage entry is archived |
| live_until_ledger_seq | The ledger sequence number at which a Persistent entry expires |
| State Archival | Stellar protocol mechanism that archives inactive Persistent storage to reduce ledger state size |
| Renewal Engine | On-chain component that computes and submits rent fee to extend TTL |
