# Axiom State Monitor

A production-ready Soroban smart contract that monitors the TTL (Time To Live) of Persistent storage entries on the Stellar network to prevent State Archival data loss.

Built for the **Drips Wave 2026** contributor program. See [STELLAR_WAVE.md](./STELLAR_WAVE.md) to earn XLM by contributing.

## What it does

Soroban Persistent storage entries expire if their TTL reaches zero — they become archived and unreadable, then permanently deleted. Axiom State Monitor watches your critical storage keys and alerts before that happens.

- **Storage Watcher** — queries `live_until_ledger_seq` and classifies each entry as `Live`, `Warning`, `Archived`, or `Dead`
- **Renewal Engine** — calculates rent fees and extends TTL via `extend_ttl`
- **Alert System** — emits structured Soroban Events (`ttl_warning`, `archived`, `dead`) for off-chain indexing

## State Lifecycle

```
Live ──(TTL expires)──► Archived ──(grace period ends)──► Dead
 ▲                          │
 └──(rent paid / TTL ext)───┘
```

## Quickstart

### Prerequisites

- Rust 1.75+ with `wasm32-unknown-unknown` target
- [`stellar-cli`](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)
- `wasm-opt` (binaryen) for optimization

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli
```

### Build & Optimize

```bash
make build optimize
```

### Deploy to Testnet

```bash
ADMIN_SECRET=<your-secret-key> make deploy
```

### Run Tests

```bash
make test
```

### Fee Benchmark

```bash
bash scripts/bench_fee.sh <CONTRACT_ID>
```

## Contract Interface

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(admin, threshold)` | — | One-time setup |
| `watch(key)` | Admin | Add a key to the watch list |
| `unwatch(key)` | Admin | Remove a key from the watch list |
| `check_entry(key)` | Public | Query TTL state for a single key |
| `check_all()` | Public | Batch TTL check across all watched keys |
| `extend_ttl(key, ledgers)` | Admin | Extend TTL by N ledgers |
| `calc_fee(size, ledgers)` | Public | Estimate rent fee in stroops |
| `set_threshold(ledgers)` | Admin | Update warning threshold |

## Repo Structure

```
├── contracts/axiom-state-monitor/   # Soroban Rust contract
├── scripts/                         # Linux benchmarking scripts
├── specs/                           # Requirements, design, and task docs
├── Makefile                         # Build / deploy automation
└── STELLAR_WAVE.md                  # Drips Wave contributor guide
```

## Contributing

See [specs/tasks.md](./specs/tasks.md) for open Wave issues and [STELLAR_WAVE.md](./STELLAR_WAVE.md) for the Fix, Merge, Earn process.

## License

MIT
