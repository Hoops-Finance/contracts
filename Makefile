CRATES := common adapter-interface adapters/* router account account_deployer

.PHONY: build fmt clean

build:
	@for c in $(CRATES); do \
	  echo "→ $$c"; \
	  cargo build --manifest-path $$c/Cargo.toml --target wasm32-unknown-unknown --release; \
	done

fmt:
	cargo fmt --all

clean:
	cargo clean
