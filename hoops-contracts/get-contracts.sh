#!/bin/bash
# get-contracts.sh
# Copies all built Phoenix WASMs from /home/silence/hoops-target/wasm32v1-none/release into the correct external_contracts/phoenix folders in hoops-contracts.

set -e

SRC_DIR="/home/silence/hoops-target/wasm32v1-none/release"
DST_DIR="$(dirname "$0")/external_contracts/phoenix"

# List of Phoenix contract WASMs to copy (add more as needed)
declare -A CONTRACTS=(
  [phoenix_stake.wasm]="stake"
  [phoenix_token.wasm]="token"
  [phoenix_factory.wasm]="factory"
  [phoenix_pool.wasm]="pool"
  [phoenix_pool_stable.wasm]="pool_stable"
  [phoenix_multihop.wasm]="multihop"
)

for wasm in "${!CONTRACTS[@]}"; do
  src="$SRC_DIR/$wasm"
  dst="$DST_DIR/${CONTRACTS[$wasm]}/$wasm"
  if [ -f "$src" ]; then
    mkdir -p "$(dirname "$dst")"
    cp "$src" "$dst"
    echo "Copied $src -> $dst"
  else
    echo "Warning: $src not found, skipping."
  fi
done

# Copy comet contracts.wasm to comet-pool.wasm
COMET_SRC="$SRC_DIR/contracts.wasm"
COMET_DST="$(dirname "$0")/external_contracts/comet/comet-pool.wasm"
if [ -f "$COMET_SRC" ]; then
  mkdir -p "$(dirname "$COMET_DST")"
  cp "$COMET_SRC" "$COMET_DST"
  echo "Copied $COMET_SRC -> $COMET_DST"
else
  echo "Warning: $COMET_SRC not found, skipping."
fi

# Copy comet factory.wasm to comet-factory.wasm
COMET_FACTORY_SRC="$SRC_DIR/factory.wasm"
COMET_FACTORY_DST="$(dirname "$0")/external_contracts/comet/comet-factory.wasm"
if [ -f "$COMET_FACTORY_SRC" ]; then
  mkdir -p "$(dirname "$COMET_FACTORY_DST")"
  cp "$COMET_FACTORY_SRC" "$COMET_FACTORY_DST"
  echo "Copied $COMET_FACTORY_SRC -> $COMET_FACTORY_DST"
else
  echo "Warning: $COMET_FACTORY_SRC not found, skipping."
fi

echo "All available Phoenix and Comet WASMs copied."
