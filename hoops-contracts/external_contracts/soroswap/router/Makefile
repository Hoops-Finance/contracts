default: build

all: test

test: build
	cargo test

build:
	$(MAKE) -C ../token || break;
	$(MAKE) -C ../pair || break;
	$(MAKE) -C ../factory || break;
	cargo build --target wasm32v1-none --release
	soroban contract optimize --wasm bytecodes/soroswap_router.wasm
	@ls -l bytecodes/*.wasm
fmt:
	cargo fmt --all --check

clean:
	cargo clean
