ifeq (,$(BUILD_FLAGS))
  DEPS = ../factory ../pool
endif

default: all

all: lint build test

test:
	$(MAKE) -C ../factory build || break;
	$(MAKE) -C ../pool build || break;
	cargo test

build:
	@for dir in $(DEPS) ; do \
		$(MAKE) -C $$dir build || break; \
	done
	cargo build --target wasm32v1-none --release

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
