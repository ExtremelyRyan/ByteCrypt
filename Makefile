.PHONY: run build check test clippy fmt lint cic clean

# run and compile
run:
	cargo run

build:
	cargo build

build-release:
	cargo build --release

# test and lint
check:
	cargo check --all

test:
	cargo test --workspace

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt --all -- --check

# utility
lint: fmt clippy

## can i commit?
cic: test lint

clean: 
	cargo clean