default: all

all: lint build test

test: build # because of token dependency
	cargo test

build:
	$(MAKE) -C ../stake build || break;
	$(MAKE) -C ../token build || break;
	cargo build --target wasm32v1-none --release

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings -A clippy::too_many_arguments

clean:
	cargo clean
