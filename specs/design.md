# Axiom State Monitor — Design Document

## State Archival Lifecycle

Soroban Persistent storage entries move through three states:

```
Live ──(TTL expires)──► Archived ──(grace period ends)──► Dead
 ▲                          │
 └──(rent paid / TTL ext)───┘  (only possible from Archived, not Dead)
```

### Live
- Entry exists in the active ledger state.
- `live_until_ledger_seq` is a future ledger sequence number.
- Readable and writable by contracts.
- The Storage Watcher polls this value each invocation.

### Archived
- TTL reached zero; entry is evicted from active ledger state.
- Data is preserved off-chain via Stellar's archival proofs.
- Contract reads/writes to this key will FAIL until restored.
- Restoration requires submitting a `RestoreFootprint` operation with the correct rent fee.
- The Renewal Engine detects this state and blocks renewal, emitting `archived_entry`.

### Dead
- Grace period after archival has elapsed with no restoration.
- Entry is permanently deleted; data is unrecoverable.
- No renewal or restoration is possible.
- The system emits `entry_dead` and halts all operations on that key.

---

## Component Architecture

```
┌─────────────────────────────────────────────┐
│              Axiom State Monitor             │
│                                              │
│  ┌──────────────────┐  ┌──────────────────┐ │
│  │  Storage Watcher │  │  Renewal Engine  │ │
│  │                  │  │                  │ │
│  │ - get_ttl(key)   │  │ - calc_fee(n)    │ │
│  │ - classify_state │  │ - extend_ttl(n)  │ │
│  └────────┬─────────┘  └────────┬─────────┘ │
│           │                     │            │
│           └──────────┬──────────┘            │
│                      ▼                       │
│             ┌────────────────┐               │
│             │  Alert System  │               │
│             │                │               │
│             │ - emit events  │               │
│             │ - ttl_warning  │               │
│             │ - ttl_critical │               │
│             │ - archived     │               │
│             └────────────────┘               │
└─────────────────────────────────────────────┘
```

## Storage Layout

| Key | Type | Description |
|-----|------|-------------|
| `WatchedKeys` | Persistent Vec | List of storage keys under monitoring |
| `TTLThreshold` | Instance | Ledger count before warning is emitted |
| `AdminAddress` | Instance | Address authorized to call renewal |

## Rent Fee Calculation

Fee = `entry_size_bytes * fee_per_byte_per_ledger * ledgers_to_extend`

The `fee_per_byte_per_ledger` is sourced from the current network ledger configuration via `env.ledger()`.

## Security Model

- All mutating functions require `admin.require_auth()`.
- TTL queries are read-only and permissionless.
- Events are emitted with structured topics for off-chain indexing.
