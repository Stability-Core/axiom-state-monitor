#!/usr/bin/env bash
# bench_fee.sh — Estimate TTL renewal fees on Stellar Testnet
# Usage: bash scripts/bench_fee.sh [CONTRACT_ID]
set -euo pipefail

CONTRACT_ID="${1:-}"
NETWORK="${NETWORK:-testnet}"
ENTRY_SIZE_BYTES=1024
LEDGERS_TO_EXTEND=100000

if [[ -z "$CONTRACT_ID" ]]; then
  echo "Usage: $0 <CONTRACT_ID>"
  echo "  NETWORK env var defaults to 'testnet'"
  exit 1
fi

echo "=== Axiom State Monitor — Fee Benchmark ==="
echo "Network:          $NETWORK"
echo "Contract:         $CONTRACT_ID"
echo "Entry size:       ${ENTRY_SIZE_BYTES} bytes"
echo "Ledgers to extend: ${LEDGERS_TO_EXTEND}"
echo ""

stellar contract invoke \
  --id "$CONTRACT_ID" \
  --network "$NETWORK" \
  --source-account default \
  -- calc_fee \
  --entry_size_bytes "$ENTRY_SIZE_BYTES" \
  --ledgers "$LEDGERS_TO_EXTEND"
