default: build

all: test

test: build
	cargo test

build:
	cargo build --target wasm32v1-none --release

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --tests -- -D warnings

clean:
	cargo clean
