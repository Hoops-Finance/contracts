default: build

all: test

test: build
	cargo test

build:
	$(MAKE) -C ../token || break;
	$(MAKE) -C ../pair || break;
	$(MAKE) -C ../factory || break;
	cp ../pair/bytecodes/soroswap_pair.wasm ./src  # Copy pair wasm to library root
	cargo build --target wasm32v1-none --release
	soroban contract optimize --wasm bytecodes/soroswap_library.wasm
	@ls -l bytecodes/*.wasm
fmt:
	cargo fmt --all --check

clean:
	cargo clean
