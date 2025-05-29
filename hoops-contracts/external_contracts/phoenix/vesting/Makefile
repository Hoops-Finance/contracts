default: all

all: lint build test

test: build
	cargo test --all-features

build:
	$(MAKE) -C ../token build || break;
	cargo build --all-features --target wasm32v1-none --release

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean
