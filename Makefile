# Axiom State Monitor — Makefile for Ubuntu/Linux
# Requires: stellar-cli, cargo, wasm-opt (binaryen)

CONTRACT     := axiom-state-monitor
WASM_TARGET  := target/wasm32-unknown-unknown/release/$(CONTRACT).wasm
WASM_OPT     := target/wasm32-unknown-unknown/release/$(CONTRACT).optimized.wasm
NETWORK      ?= testnet
ADMIN_SECRET ?= $(shell cat .admin_secret 2>/dev/null)

.PHONY: all build optimize deploy test lint clean

all: build optimize

## Build the contract WASM
build:
	cargo build --target wasm32-unknown-unknown --release \
		-p $(CONTRACT)

## Optimize WASM with wasm-opt (binaryen)
optimize: build
	wasm-opt -Oz --strip-debug \
		$(WASM_TARGET) \
		-o $(WASM_OPT)
	@echo "Optimized WASM: $(WASM_OPT)"
	@wc -c $(WASM_OPT)

## Deploy to Stellar network (default: testnet)
deploy: optimize
	stellar contract deploy \
		--wasm $(WASM_OPT) \
		--source $(ADMIN_SECRET) \
		--network $(NETWORK)

## Run unit tests
test:
	cargo test -p $(CONTRACT) -- --nocapture

## Lint with clippy
lint:
	cargo clippy -p $(CONTRACT) -- -D warnings

## Format check
fmt-check:
	cargo fmt -p $(CONTRACT) -- --check

## Benchmark: estimate fee for a 1KB entry extended by 100k ledgers
bench:
	@bash scripts/bench_fee.sh

## Clean build artifacts
clean:
	cargo clean
