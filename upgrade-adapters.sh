#!/bin/bash

# Hoops Adapters - Build & Redeploy Script
# Builds adapter WASMs and redeploys to testnet.
#
# When upgrade is available (CoreConfig set), uses in-place upgrade.
# Otherwise, deploys a fresh instance, re-initializes, re-registers pools,
# and updates the Router mapping.
#
# Prerequisites:
#   - stellar CLI installed
#   - 'deployer' key configured: stellar keys generate deployer --network testnet
#   - testnet-suite.env exists with current addresses
#
# Usage:
#   bash upgrade-adapters.sh              # Redeploy ALL adapters
#   bash upgrade-adapters.sh aqua         # Redeploy only Aqua adapter
#   bash upgrade-adapters.sh aqua comet   # Redeploy Aqua and Comet

set -e

echo "========================================================"
echo "  Hoops Adapters - Build & Redeploy to Testnet"
echo "========================================================"
echo ""

cd hoops-contracts

# ---- Load addresses ----
if [ -f "../testnet-suite.env" ]; then
    source ../testnet-suite.env
    echo "Loaded addresses from testnet-suite.env"
else
    echo "ERROR: testnet-suite.env not found. Run deploy-testnet-suite.sh first."
    exit 1
fi

# ---- Verify deployer key ----
if ! stellar keys ls | grep -q "deployer"; then
    echo "ERROR: 'deployer' key not found. Run: stellar keys generate deployer --network testnet"
    exit 1
fi
DEPLOYER=$(stellar keys address deployer)
echo "Deployer: $DEPLOYER"

# ---- Helpers ----
invoke() {
    stellar contract invoke --id "$1" --source deployer --network testnet -- "${@:2}"
}

deploy_wasm() {
    stellar contract deploy --wasm "$1" --source deployer --network testnet
}

install_wasm() {
    stellar contract install --wasm "$1" --source deployer --network testnet 2>/dev/null \
        || stellar contract upload --wasm "$1" --source deployer --network testnet
}

# ---- Determine which adapters to redeploy ----
if [ $# -eq 0 ]; then
    TARGETS="aqua comet phoenix soroswap"
    echo "Redeploying ALL adapters: $TARGETS"
else
    TARGETS="$@"
    echo "Redeploying: $TARGETS"
fi

# ---- Build adapters ----
echo ""
echo "Building adapter WASMs..."
for target in $TARGETS; do
    case $target in
        aqua)     PKG="aqua-adapter" ;;
        comet)    PKG="comet-adapter" ;;
        phoenix)  PKG="phoenix-adapter" ;;
        soroswap) PKG="soroswap-adapter" ;;
        *)
            echo "ERROR: Unknown adapter '$target'. Valid: aqua, comet, phoenix, soroswap"
            exit 1
            ;;
    esac
    echo "  Building $PKG..."
    cargo build --release --target wasm32-unknown-unknown -p "$PKG" 2>&1 | tail -1
done
echo "Build complete."

# ---- Track new addresses (simple vars for bash 3.2 compat) ----
NEW_AQUA="" NEW_COMET="" NEW_PHOENIX="" NEW_SOROSWAP=""

get_new_addr() {
    case $1 in
        aqua)     echo "$NEW_AQUA" ;;
        comet)    echo "$NEW_COMET" ;;
        phoenix)  echo "$NEW_PHOENIX" ;;
        soroswap) echo "$NEW_SOROSWAP" ;;
    esac
}

set_new_addr() {
    case $1 in
        aqua)     NEW_AQUA="$2" ;;
        comet)    NEW_COMET="$2" ;;
        phoenix)  NEW_PHOENIX="$2" ;;
        soroswap) NEW_SOROSWAP="$2" ;;
    esac
}

# ---- Redeploy each adapter ----
echo ""
for target in $TARGETS; do
    case $target in
        aqua)
            WASM_NAME="aqua_adapter"
            OLD_ADDR="$AQUA_ADAPTER"
            ADAPTER_ID=0
            AMM_ADDR="$AQUA_ROUTER_EXT"
            ;;
        comet)
            WASM_NAME="comet_adapter"
            OLD_ADDR="$COMET_ADAPTER"
            ADAPTER_ID=1
            AMM_ADDR="$COMET_POOL_EXT"
            ;;
        phoenix)
            WASM_NAME="phoenix_adapter"
            OLD_ADDR="$PHOENIX_ADAPTER"
            ADAPTER_ID=2
            AMM_ADDR="$PHOENIX_POOL_EXT"
            ;;
        soroswap)
            WASM_NAME="soroswap_adapter"
            OLD_ADDR="$SOROSWAP_ADAPTER"
            ADAPTER_ID=3
            AMM_ADDR="$SOROSWAP_ROUTER_EXT"
            ;;
    esac

    WASM_SRC="target/wasm32-unknown-unknown/release/${WASM_NAME}.wasm"
    WASM_DST="bytecodes/${WASM_NAME}.wasm"

    echo "================================================================"
    echo "  Redeploying $target adapter (ID=$ADAPTER_ID)"
    echo "  Old address: $OLD_ADDR"
    echo "================================================================"

    # Copy WASM to bytecodes
    cp "$WASM_SRC" "$WASM_DST"
    echo "  Copied WASM ($(wc -c < "$WASM_DST") bytes)"

    # Try in-place upgrade first
    echo "  Attempting in-place upgrade..."
    NEW_HASH=$(install_wasm "$WASM_DST")
    echo "  WASM hash: $NEW_HASH"

    if stellar contract invoke \
        --id "$OLD_ADDR" \
        --source deployer \
        --network testnet \
        -- upgrade --new_wasm_hash "$NEW_HASH" 2>/dev/null; then
        echo "  In-place upgrade succeeded! Address unchanged."
        set_new_addr "$target" "$OLD_ADDR"
    else
        echo "  In-place upgrade not available. Deploying fresh instance..."

        # Deploy fresh contract
        NEW_ADDR=$(deploy_wasm "$WASM_DST" | tr -d '"')
        echo "  New address: $NEW_ADDR"
        set_new_addr "$target" "$NEW_ADDR"

        # Initialize adapter
        echo "  Initializing (amm_id=$ADAPTER_ID, amm_addr=$AMM_ADDR)..."
        invoke "$NEW_ADDR" initialize \
            --amm_id $ADAPTER_ID \
            --amm_addr "$AMM_ADDR"
        echo "  Initialized."

        # Re-register pools
        case $target in
            aqua)
                if [ -n "$AQUA_POOL_ADDRESS" ] && [ -n "$AQUA_LP_TOKEN" ]; then
                    echo "  Registering Aqua pool..."
                    invoke "$NEW_ADDR" set_pool_for_tokens \
                        --tokens "[\"$USDC_TOKEN\",\"$XLM_SAC\"]" \
                        --info "{\"pool_address\":\"$AQUA_POOL_ADDRESS\",\"lp_token_address\":\"$AQUA_LP_TOKEN\"}"
                    echo "  Pool registered."
                fi
                ;;
            comet)
                if [ -n "$COMET_POOL_EXT" ]; then
                    echo "  Registering Comet pool..."
                    invoke "$NEW_ADDR" set_pool_for_tokens \
                        --tokens "[\"$USDC_TOKEN\",\"$XLM_SAC\"]" \
                        --pool "$COMET_POOL_EXT"
                    echo "  Pool registered."
                fi
                ;;
            phoenix|soroswap)
                echo "  No pool registration needed ($target uses factory/single-pool lookup)."
                ;;
        esac

        # Update Router mapping
        echo "  Updating Router adapter mapping (ID=$ADAPTER_ID -> $NEW_ADDR)..."
        invoke "$ROUTER" add_adapter \
            --adapter_id $ADAPTER_ID \
            --adapter_address "$NEW_ADDR"
        echo "  Router updated."
    fi
    echo ""
done

# ---- Summary ----
echo "========================================================"
echo "  Redeployment complete!"
echo "========================================================"
echo ""

CHANGED=false
for target in $TARGETS; do
    case $target in
        aqua)     OLD="$AQUA_ADAPTER" ;;
        comet)    OLD="$COMET_ADAPTER" ;;
        phoenix)  OLD="$PHOENIX_ADAPTER" ;;
        soroswap) OLD="$SOROSWAP_ADAPTER" ;;
    esac
    NEW=$(get_new_addr "$target")
    if [ "$OLD" != "$NEW" ]; then
        echo "  $target: $OLD -> $NEW  (CHANGED)"
        CHANGED=true
    else
        echo "  $target: $OLD  (unchanged)"
    fi
done

if [ "$CHANGED" = true ]; then
    echo ""
    echo "WARNING: Adapter addresses changed! Update these files:"
    echo "  - testnet-suite.env"
    echo "  - hoops_dashboard-ui/.env.local"
    echo "  - hoops_dashboard-ui/.env.testnet"
    echo "  - hoops_dashboard-ui/lib/hoops-contracts.ts (defaults)"
    echo "  - hoops_dashboard-ui/lib/contracts.config.ts (defaults)"

    # Auto-update testnet-suite.env adapter lines
    echo ""
    echo "Auto-updating testnet-suite.env..."
    for target in $TARGETS; do
        NEW=$(get_new_addr "$target")
        case $target in
            aqua)     sed -i '' "s|^export AQUA_ADAPTER=.*|export AQUA_ADAPTER=$NEW|" ../testnet-suite.env ;;
            comet)    sed -i '' "s|^export COMET_ADAPTER=.*|export COMET_ADAPTER=$NEW|" ../testnet-suite.env ;;
            phoenix)  sed -i '' "s|^export PHOENIX_ADAPTER=.*|export PHOENIX_ADAPTER=$NEW|" ../testnet-suite.env ;;
            soroswap) sed -i '' "s|^export SOROSWAP_ADAPTER=.*|export SOROSWAP_ADAPTER=$NEW|" ../testnet-suite.env ;;
        esac
    done
    echo "  testnet-suite.env updated."
fi

echo ""
echo "Verify on Stellar Expert:"
for target in $TARGETS; do
    ADDR=$(get_new_addr "$target")
    printf "  %-10s https://stellar.expert/explorer/testnet/contract/%s\n" "$target:" "$ADDR"
done
echo ""
