ifeq (,$(BUILD_FLAGS))
  DEPS = ../stake ../pool ../pool_stable
endif

default: all

all: lint build test

test: build # because of token dependency
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
	cargo clippy --all-targets -- -D warnings -A clippy::too_many_arguments

clean:
	cargo clean
