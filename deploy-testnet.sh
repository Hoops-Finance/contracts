#!/bin/bash

# Hoops Contracts - Testnet Deployment Script
# This script deploys all contracts to Stellar testnet
# Generated: February 11, 2026

set -e  # Exit on any error

echo "🚀 Starting Hoops Contracts Deployment to Testnet"
echo "=================================================="
echo ""

# Navigate to contracts directory
cd hoops-contracts

# Check if deployer key exists
if ! stellar keys ls | grep -q "deployer"; then
    echo "❌ Error: 'deployer' key not found"
    echo "Run: stellar keys generate deployer --network testnet"
    exit 1
fi

DEPLOYER_ADDRESS=$(stellar keys address deployer)
echo "📍 Deployer Address: $DEPLOYER_ADDRESS"
echo ""

# Create output file for contract addresses
OUTPUT_FILE="../deployed-contracts-testnet.json"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

echo "📝 Deployment Log"
echo "================="
echo ""

# Deploy Adapters
echo "1️⃣  Deploying Aqua Adapter..."
AQUA_ADAPTER=$(stellar contract deploy \
    --wasm bytecodes/aqua_adapter.wasm \
    --source deployer \
    --network testnet)
echo "   ✅ Aqua Adapter: $AQUA_ADAPTER"

echo "2️⃣  Deploying Comet Adapter..."
COMET_ADAPTER=$(stellar contract deploy \
    --wasm bytecodes/comet_adapter.wasm \
    --source deployer \
    --network testnet)
echo "   ✅ Comet Adapter: $COMET_ADAPTER"

echo "3️⃣  Deploying Phoenix Adapter..."
PHOENIX_ADAPTER=$(stellar contract deploy \
    --wasm bytecodes/phoenix_adapter.wasm \
    --source deployer \
    --network testnet)
echo "   ✅ Phoenix Adapter: $PHOENIX_ADAPTER"

echo "4️⃣  Deploying Soroswap Adapter..."
SOROSWAP_ADAPTER=$(stellar contract deploy \
    --wasm bytecodes/soroswap_adapter.wasm \
    --source deployer \
    --network testnet)
echo "   ✅ Soroswap Adapter: $SOROSWAP_ADAPTER"

echo ""
echo "5️⃣  Deploying Router..."
ROUTER=$(stellar contract deploy \
    --wasm bytecodes/hoops_router.wasm \
    --source deployer \
    --network testnet)
echo "   ✅ Router: $ROUTER"

echo "6️⃣  Deploying Account Deployer..."
ACCOUNT_DEPLOYER=$(stellar contract deploy \
    --wasm bytecodes/hoops_account_deployer.wasm \
    --source deployer \
    --network testnet)
echo "   ✅ Account Deployer: $ACCOUNT_DEPLOYER"

echo ""
echo "🔧 Initializing Contracts..."
echo "============================"
echo ""

# Initialize Router
echo "7️⃣  Initializing Router..."
stellar contract invoke \
    --id $ROUTER \
    --source deployer \
    --network testnet \
    -- \
    initialize \
    --admin $DEPLOYER_ADDRESS > /dev/null
echo "   ✅ Router initialized with admin: $DEPLOYER_ADDRESS"

# Register adapters with router
echo "8️⃣  Registering Adapters with Router..."

echo "   Adding Aqua (ID: 0)..."
stellar contract invoke \
    --id $ROUTER \
    --source deployer \
    --network testnet \
    -- \
    add_adapter \
    --adapter_id 0 \
    --adapter_address $AQUA_ADAPTER > /dev/null

echo "   Adding Comet (ID: 1)..."
stellar contract invoke \
    --id $ROUTER \
    --source deployer \
    --network testnet \
    -- \
    add_adapter \
    --adapter_id 1 \
    --adapter_address $COMET_ADAPTER > /dev/null

echo "   Adding Phoenix (ID: 2)..."
stellar contract invoke \
    --id $ROUTER \
    --source deployer \
    --network testnet \
    -- \
    add_adapter \
    --adapter_id 2 \
    --adapter_address $PHOENIX_ADAPTER > /dev/null

echo "   Adding Soroswap (ID: 3)..."
stellar contract invoke \
    --id $ROUTER \
    --source deployer \
    --network testnet \
    -- \
    add_adapter \
    --adapter_id 3 \
    --adapter_address $SOROSWAP_ADAPTER > /dev/null
echo "   ✅ All adapters registered"

echo ""
echo "9️⃣  Deploying Test Account..."
TEST_ACCOUNT=$(stellar contract deploy \
    --wasm bytecodes/hoops_account.wasm \
    --source deployer \
    --network testnet)
echo "   ✅ Test Account: $TEST_ACCOUNT"

echo "🔟 Initializing Test Account..."
stellar contract invoke \
    --id $TEST_ACCOUNT \
    --source deployer \
    --network testnet \
    -- \
    initialize \
    --owner $DEPLOYER_ADDRESS \
    --router $ROUTER > /dev/null
echo "   ✅ Test Account initialized"

echo ""
echo "💾 Saving Contract Addresses..."
echo "==============================="

# Save as JSON
cat > $OUTPUT_FILE << EOF
{
  "network": "testnet",
  "deployed_at": "$TIMESTAMP",
  "deployer_address": "$DEPLOYER_ADDRESS",
  "contracts": {
    "adapters": {
      "aqua": {
        "id": 0,
        "address": "$AQUA_ADAPTER"
      },
      "comet": {
        "id": 1,
        "address": "$COMET_ADAPTER"
      },
      "phoenix": {
        "id": 2,
        "address": "$PHOENIX_ADAPTER"
      },
      "soroswap": {
        "id": 3,
        "address": "$SOROSWAP_ADAPTER"
      }
    },
    "core": {
      "router": "$ROUTER",
      "account_deployer": "$ACCOUNT_DEPLOYER"
    },
    "test": {
      "account": "$TEST_ACCOUNT"
    }
  }
}
EOF

# Also save as environment variables file for easy sourcing
cat > ../deployed-contracts-testnet.env << EOF
# Hoops Contracts - Testnet Addresses
# Deployed: $TIMESTAMP

export NETWORK=testnet
export DEPLOYER_ADDRESS=$DEPLOYER_ADDRESS

# Adapters
export AQUA_ADAPTER=$AQUA_ADAPTER
export COMET_ADAPTER=$COMET_ADAPTER
export PHOENIX_ADAPTER=$PHOENIX_ADAPTER
export SOROSWAP_ADAPTER=$SOROSWAP_ADAPTER

# Core Contracts
export ROUTER=$ROUTER
export ACCOUNT_DEPLOYER=$ACCOUNT_DEPLOYER

# Test Account
export TEST_ACCOUNT=$TEST_ACCOUNT
EOF

echo "   ✅ Saved to: $OUTPUT_FILE"
echo "   ✅ Saved to: ../deployed-contracts-testnet.env"

echo ""
echo "✅ Deployment Complete!"
echo "======================"
echo ""
echo "📋 Summary:"
echo "  - Aqua Adapter:      $AQUA_ADAPTER"
echo "  - Comet Adapter:     $COMET_ADAPTER"
echo "  - Phoenix Adapter:   $PHOENIX_ADAPTER"
echo "  - Soroswap Adapter:  $SOROSWAP_ADAPTER"
echo "  - Router:            $ROUTER"
echo "  - Account Deployer:  $ACCOUNT_DEPLOYER"
echo "  - Test Account:      $TEST_ACCOUNT"
echo ""
echo "🔍 Verify on Stellar Expert:"
echo "   https://stellar.expert/explorer/testnet/contract/$ROUTER"
echo ""
echo "📝 Contract addresses saved to:"
echo "   - $OUTPUT_FILE (JSON format)"
echo "   - ../deployed-contracts-testnet.env (shell variables)"
echo ""
echo "🎉 Ready for Day 3: UI Development!"
