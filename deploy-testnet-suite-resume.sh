#!/bin/bash

# Hoops Testnet Suite - RESUME from step 2.7
# Continues deployment after Phase 1 + Phase 2 (steps 2.1-2.6) already succeeded.
# The initial run failed at step 2.7 due to expiration_ledger being too large.
#
# Usage: bash deploy-testnet-suite-resume.sh

set -e

echo "========================================================"
echo "  Hoops Testnet Suite - RESUME from step 2.7"
echo "========================================================"
echo ""

cd hoops-contracts

# ---- Config ----
DEPLOYER=$(stellar keys address deployer)
echo "Deployer: $DEPLOYER"

# Load existing Hoops contract addresses
source ../deployed-contracts-testnet.env
echo "Loaded Hoops addresses from deployed-contracts-testnet.env"

# ---- Already deployed addresses (from previous partial run) ----
XLM_SAC=CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
USDC_TOKEN=CAFOFEFNIHJIMC2LUSQJ2VBW4ROSB2EZ3WPBAVW5KZK5ZEBOLPKPXU5C
SOROSWAP_FACTORY=CB2W7TI4MD3HFQVPP3D5JBWHIJC3ORA46JOP6GLPXRXWVLG257VDIYNY
SOROSWAP_ROUTER=CDRRGJJ45RJFCYGVNXYOW2ALTXMAUOC7FVRA4G7PGTUS43YQ4RSHVWLU
SOROSWAP_PAIR=CDMO5NO22XX6KM6DWVEVXDVUNCI23WPDO62ZRSTNCMSIMITXWUABOMAR
SOROSWAP_PAIR_HASH=""  # not needed for resume

echo "Using previously deployed:"
echo "  XLM SAC:          $XLM_SAC"
echo "  USDC:             $USDC_TOKEN"
echo "  Soroswap Factory: $SOROSWAP_FACTORY"
echo "  Soroswap Router:  $SOROSWAP_ROUTER"
echo "  Soroswap Pair:    $SOROSWAP_PAIR"

# Amounts (7 decimals = stroops)
POOL_USDC=1000000000                 # 100 USDC per pool
POOL_XLM=5000000000                  # 500 XLM per pool (1 USDC ~ 5 XLM)
APPROVE_AMOUNT=99999999999999        # Large approval amount
DEADLINE=9999999999                  # Far future deadline

TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
OUTPUT_FILE="../testnet-suite.env"
OUTPUT_JSON="../testnet-suite.json"

# Get current ledger for approval expiration (~30 days)
CURRENT_LEDGER=$(curl -s https://horizon-testnet.stellar.org/ | python3 -c "import sys,json; print(json.load(sys.stdin)['core_latest_ledger'])")
EXPIRATION_LEDGER=$((CURRENT_LEDGER + 535000))
echo "Current ledger: $CURRENT_LEDGER, approval expiration: $EXPIRATION_LEDGER"

# Helpers
invoke() {
    stellar contract invoke --id "$1" --source deployer --network testnet -- "${@:2}"
}

deploy() {
    stellar contract deploy --wasm "$1" --source deployer --network testnet
}

install_wasm() {
    stellar contract install --wasm "$1" --source deployer --network testnet
}

echo ""
echo "================================================================"
echo "  PHASE 2 (continued): Soroswap - Approve + Add Liquidity"
echo "================================================================"
echo ""

echo "2.7 Approving tokens for Soroswap Router..."
invoke $USDC_TOKEN approve \
    --from $DEPLOYER \
    --spender $SOROSWAP_ROUTER \
    --amount $APPROVE_AMOUNT \
    --expiration_ledger $EXPIRATION_LEDGER
invoke $XLM_SAC approve \
    --from $DEPLOYER \
    --spender $SOROSWAP_ROUTER \
    --amount $APPROVE_AMOUNT \
    --expiration_ledger $EXPIRATION_LEDGER
echo "    Tokens approved"

echo "2.8 Adding initial liquidity to Soroswap..."
invoke $SOROSWAP_ROUTER add_liquidity \
    --token_a $USDC_TOKEN \
    --token_b $XLM_SAC \
    --amount_a_desired $POOL_USDC \
    --amount_b_desired $POOL_XLM \
    --amount_a_min 0 \
    --amount_b_min 0 \
    --to $DEPLOYER \
    --deadline $DEADLINE
echo "    Liquidity added: 100 USDC + 500 XLM"
echo "    SOROSWAP COMPLETE"

echo ""
echo "================================================================"
echo "  PHASE 3: Aquarius AMM"
echo "================================================================"
echo ""

echo "3.1 Installing Aqua WASMs..."
AQUA_POOL_HASH=$(install_wasm bytecodes/aqua_soroban_liquidity_pool_contract.wasm)
echo "    Pool hash: $AQUA_POOL_HASH"

AQUA_STABLE_HASH=$(install_wasm bytecodes/aqua_soroban_liquidity_pool_stableswap_contract.wasm)
echo "    StableSwap hash: $AQUA_STABLE_HASH"

AQUA_TOKEN_HASH=$(install_wasm bytecodes/soroban_token_contract_aqua.wasm)
echo "    Token hash: $AQUA_TOKEN_HASH"

echo "3.2 Deploying Aqua Plane contract..."
AQUA_PLANE=$(deploy bytecodes/aqua_soroban_liquidity_pool_plane_contract.wasm)
echo "    Plane: $AQUA_PLANE"

echo "3.3 Deploying Aqua Liquidity Calculator..."
AQUA_CALCULATOR=$(deploy bytecodes/aqua_soroban_liquidity_pool_liquidity_calculator_contract.wasm)
echo "    Calculator: $AQUA_CALCULATOR"

echo "3.4 Deploying Aqua Router..."
AQUA_ROUTER=$(deploy bytecodes/aqua_soroban_liquidity_pool_router_contract.wasm)
echo "    Router: $AQUA_ROUTER"

echo "3.5 Initializing Aqua Router admin..."
invoke $AQUA_ROUTER init_admin \
    --account $DEPLOYER
echo "    Admin set"

echo "3.6 Setting Aqua WASM hashes..."
invoke $AQUA_ROUTER set_pool_hash \
    --admin $DEPLOYER \
    --new_hash $AQUA_POOL_HASH

invoke $AQUA_ROUTER set_stableswap_pool_hash \
    --admin $DEPLOYER \
    --new_hash $AQUA_STABLE_HASH

invoke $AQUA_ROUTER set_token_hash \
    --admin $DEPLOYER \
    --new_hash $AQUA_TOKEN_HASH
echo "    WASM hashes registered"

echo "3.7 Setting Aqua privileged addresses..."
invoke $AQUA_ROUTER set_privileged_addrs \
    --admin $DEPLOYER \
    --rewards_admin $DEPLOYER \
    --operations_admin $DEPLOYER \
    --pause_admin $DEPLOYER \
    --emergency_pause_admins '["'$DEPLOYER'"]'
echo "    Privileged addresses set"

echo "3.7a Initializing Aqua Plane admin..."
invoke $AQUA_PLANE init_admin \
    --account $DEPLOYER
echo "    Plane admin set"

echo "3.7b Setting pools plane on Aqua Router..."
invoke $AQUA_ROUTER set_pools_plane \
    --admin $DEPLOYER \
    --plane $AQUA_PLANE
echo "    Pools plane set"

echo "3.7c Configuring pool creation payment (free for testnet)..."
invoke $AQUA_ROUTER configure_init_pool_payment \
    --admin $DEPLOYER \
    --token $USDC_TOKEN \
    --standard_pool_amount 0 \
    --stable_pool_amount 0 \
    --to $DEPLOYER
echo "    Pool payment configured (0 fee)"

echo "3.7d Setting reward token..."
invoke $AQUA_ROUTER set_reward_token \
    --admin $DEPLOYER \
    --reward_token $USDC_TOKEN
echo "    Reward token set"

echo "3.7e Deploying Aqua Locker Feed (reward boost feed)..."
AQUA_LOCKER_FEED=$(stellar contract deploy \
    --wasm bytecodes/aqua_locker_feed.wasm \
    --source deployer \
    --network testnet \
    -- \
    --admin $DEPLOYER \
    --operations_admin $DEPLOYER \
    --emergency_admin $DEPLOYER)
echo "    Locker Feed: $AQUA_LOCKER_FEED"

echo "3.7f Setting reward boost config..."
invoke $AQUA_ROUTER set_reward_boost_config \
    --admin $DEPLOYER \
    --reward_boost_token $USDC_TOKEN \
    --reward_boost_feed $AQUA_LOCKER_FEED
echo "    Reward boost config set"

echo "3.8 Creating Aqua standard pool (USDC/XLM, 0.3% fee)..."
AQUA_POOL_RESULT=$(invoke $AQUA_ROUTER init_standard_pool \
    --user $DEPLOYER \
    --tokens '["'$USDC_TOKEN'","'$XLM_SAC'"]' \
    --fee_fraction 30)
echo "    Pool result: $AQUA_POOL_RESULT"

# Parse pool index and address from result
# init_standard_pool returns (BytesN<32>, Address) as JSON: ["hex","C..."]
AQUA_POOL_INDEX=$(echo "$AQUA_POOL_RESULT" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())[0])")
AQUA_POOL_ADDRESS=$(echo "$AQUA_POOL_RESULT" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())[1])")
echo "    Pool index: $AQUA_POOL_INDEX"
echo "    Pool address: $AQUA_POOL_ADDRESS"

echo "3.9 Approving tokens for Aqua Router..."
invoke $USDC_TOKEN approve \
    --from $DEPLOYER \
    --spender $AQUA_ROUTER \
    --amount $APPROVE_AMOUNT \
    --expiration_ledger $EXPIRATION_LEDGER
invoke $XLM_SAC approve \
    --from $DEPLOYER \
    --spender $AQUA_ROUTER \
    --amount $APPROVE_AMOUNT \
    --expiration_ledger $EXPIRATION_LEDGER
echo "    Tokens approved"

echo "3.10 Adding initial liquidity to Aqua pool..."
invoke $AQUA_ROUTER deposit \
    --user $DEPLOYER \
    --tokens '["'$USDC_TOKEN'","'$XLM_SAC'"]' \
    --pool_index $AQUA_POOL_INDEX \
    --desired_amounts '["'$POOL_USDC'","'$POOL_XLM'"]' \
    --min_shares 0
echo "    Liquidity added: 100 USDC + 500 XLM"

AQUA_LP_TOKEN="TODO_QUERY_LP_TOKEN"
echo "    AQUARIUS COMPLETE"

echo ""
echo "================================================================"
echo "  PHASE 4: Phoenix DEX"
echo "================================================================"
echo ""

echo "4.1 Installing Phoenix WASMs..."
PHX_POOL_HASH=$(install_wasm bytecodes/phoenix_pool.wasm)
echo "    Pool hash: $PHX_POOL_HASH"

PHX_STABLE_HASH=$(install_wasm bytecodes/phoenix_pool_stable.wasm)
echo "    Stable hash: $PHX_STABLE_HASH"

PHX_STAKE_HASH=$(install_wasm bytecodes/phoenix_stake.wasm)
echo "    Stake hash: $PHX_STAKE_HASH"

PHX_TOKEN_HASH=$(install_wasm bytecodes/soroban_token_contract_phoenix.wasm)
echo "    Token hash: $PHX_TOKEN_HASH"

PHX_MULTIHOP_HASH=$(install_wasm bytecodes/phoenix_multihop.wasm)
echo "    Multihop hash: $PHX_MULTIHOP_HASH"

echo "4.2 Deploying Phoenix Factory (with constructor)..."
PHOENIX_FACTORY=$(stellar contract deploy \
    --wasm bytecodes/phoenix_factory.wasm \
    --source deployer \
    --network testnet \
    -- \
    --admin $DEPLOYER \
    --multihop_wasm_hash $PHX_MULTIHOP_HASH \
    --lp_wasm_hash $PHX_POOL_HASH \
    --stable_wasm_hash $PHX_STABLE_HASH \
    --stake_wasm_hash $PHX_STAKE_HASH \
    --token_wasm_hash $PHX_TOKEN_HASH \
    --whitelisted_accounts '["'$DEPLOYER'"]' \
    --lp_token_decimals 7)
echo "    Factory: $PHOENIX_FACTORY"

echo "4.3 Creating Phoenix USDC/XLM pool..."
PHOENIX_POOL=$(invoke $PHOENIX_FACTORY create_liquidity_pool \
    --sender $DEPLOYER \
    --lp_init_info '{"admin":"'$DEPLOYER'","swap_fee_bps":30,"fee_recipient":"'$DEPLOYER'","max_allowed_slippage_bps":5000,"default_slippage_bps":2500,"max_allowed_spread_bps":5000,"max_referral_bps":500,"token_init_info":{"token_a":"'$USDC_TOKEN'","token_b":"'$XLM_SAC'"},"stake_init_info":{"min_bond":"1000000","min_reward":"1000000","manager":"'$DEPLOYER'","max_complexity":10}}' \
    --share_token_name '"USDC-XLM LP"' \
    --share_token_symbol '"USDCXLM"' \
    --pool_type 0 \
    --default_slippage_bps 100 \
    --max_allowed_fee_bps 1000)
echo "    Pool: $PHOENIX_POOL"

echo "4.4 Approving tokens for Phoenix Pool..."
invoke $USDC_TOKEN approve \
    --from $DEPLOYER \
    --spender $PHOENIX_POOL \
    --amount $APPROVE_AMOUNT \
    --expiration_ledger $EXPIRATION_LEDGER
invoke $XLM_SAC approve \
    --from $DEPLOYER \
    --spender $PHOENIX_POOL \
    --amount $APPROVE_AMOUNT \
    --expiration_ledger $EXPIRATION_LEDGER
echo "    Tokens approved"

echo "4.5 Adding initial liquidity to Phoenix pool..."
invoke $PHOENIX_POOL provide_liquidity \
    --depositor $DEPLOYER \
    --desired_a $POOL_USDC \
    --min_a 0 \
    --desired_b $POOL_XLM \
    --min_b 0 \
    --deadline $DEADLINE \
    --auto_stake false
echo "    Liquidity added: 100 USDC + 500 XLM"
echo "    PHOENIX COMPLETE"

echo ""
echo "================================================================"
echo "  PHASE 5: Comet (Balancer-style)"
echo "================================================================"
echo ""

echo "5.1 Installing Comet Pool WASM..."
COMET_POOL_HASH=$(install_wasm bytecodes/comet_pool.wasm)
echo "    Pool hash: $COMET_POOL_HASH"

echo "5.2 Deploying Comet Factory..."
COMET_FACTORY=$(deploy bytecodes/comet_factory.wasm)
echo "    Factory: $COMET_FACTORY"

echo "5.3 Initializing Comet Factory..."
invoke $COMET_FACTORY init \
    --pool_wasm_hash $COMET_POOL_HASH
echo "    Factory initialized"

echo "5.4 Approving tokens for Comet Factory..."
invoke $USDC_TOKEN approve \
    --from $DEPLOYER \
    --spender $COMET_FACTORY \
    --amount $APPROVE_AMOUNT \
    --expiration_ledger $EXPIRATION_LEDGER
invoke $XLM_SAC approve \
    --from $DEPLOYER \
    --spender $COMET_FACTORY \
    --amount $APPROVE_AMOUNT \
    --expiration_ledger $EXPIRATION_LEDGER
echo "    Tokens approved"

echo "5.5 Creating Comet 50/50 USDC/XLM pool (with initial liquidity)..."
COMET_SALT="0000000000000000000000000000000000000000000000000000000000000001"
COMET_POOL=$(invoke $COMET_FACTORY new_c_pool \
    --salt $COMET_SALT \
    --controller $DEPLOYER \
    --tokens '["'$USDC_TOKEN'","'$XLM_SAC'"]' \
    --weights '["1000000000000000000","1000000000000000000"]' \
    --balances '["'$POOL_USDC'","'$POOL_XLM'"]' \
    --swap_fee 3000000000000000)
echo "    Pool: $COMET_POOL"
echo "    COMET COMPLETE"

echo ""
echo "================================================================"
echo "  PHASE 6: Initialize Hoops Adapters"
echo "================================================================"
echo ""

if [ -z "$AQUA_ADAPTER" ]; then
    echo "WARNING: Hoops adapter addresses not found. Skipping adapter init."
    echo "Run deploy-testnet.sh first, then re-run this script."
else
    echo "6.1 Initializing Soroswap Adapter (ID=3)..."
    invoke $SOROSWAP_ADAPTER initialize \
        --amm_id 3 \
        --amm_addr $SOROSWAP_ROUTER || echo "    (may already be initialized)"

    echo "6.2 Initializing Aqua Adapter (ID=0)..."
    invoke $AQUA_ADAPTER initialize \
        --amm_id 0 \
        --amm_addr $AQUA_ROUTER || echo "    (may already be initialized)"

    echo "6.3 Initializing Phoenix Adapter (ID=2)..."
    invoke $PHOENIX_ADAPTER initialize \
        --amm_id 2 \
        --amm_addr $PHOENIX_POOL || echo "    (may already be initialized)"

    echo "6.4 Initializing Comet Adapter (ID=1)..."
    invoke $COMET_ADAPTER initialize \
        --amm_id 1 \
        --amm_addr $COMET_POOL || echo "    (may already be initialized)"

    echo "    Adapters initialized"
fi

echo ""
echo "================================================================"
echo "  PHASE 7: Register Pools with Adapters"
echo "================================================================"
echo ""

if [ -n "$AQUA_ADAPTER" ]; then
    echo "7.1 Registering pool with Aqua Adapter..."
    invoke $AQUA_ADAPTER set_pool_for_tokens \
        --tokens '["'$USDC_TOKEN'","'$XLM_SAC'"]' \
        --info '{"pool_address":"'$AQUA_POOL_ADDRESS'","lp_token_address":"'$AQUA_LP_TOKEN'"}' \
        || echo "    (registration may need LP token address)"

    echo "7.2 Registering pool with Comet Adapter..."
    invoke $COMET_ADAPTER set_pool_for_tokens \
        --tokens '["'$USDC_TOKEN'","'$XLM_SAC'"]' \
        --pool $COMET_POOL

    echo "    Pools registered"
    echo "    Note: Soroswap uses factory lookup, Phoenix uses single pool - no registration needed"
fi

echo ""
echo "================================================================"
echo "  PHASE 8: Save Addresses"
echo "================================================================"
echo ""

cat > $OUTPUT_FILE << EOF
# Hoops Testnet Suite - All Addresses
# Deployed: $TIMESTAMP

# ---- Tokens ----
export XLM_SAC=$XLM_SAC
export USDC_TOKEN=$USDC_TOKEN

# ---- Soroswap ----
export SOROSWAP_FACTORY=$SOROSWAP_FACTORY
export SOROSWAP_ROUTER_EXT=$SOROSWAP_ROUTER
export SOROSWAP_PAIR=$SOROSWAP_PAIR
export SOROSWAP_PAIR_HASH=$SOROSWAP_PAIR_HASH

# ---- Aquarius ----
export AQUA_ROUTER_EXT=$AQUA_ROUTER
export AQUA_PLANE=$AQUA_PLANE
export AQUA_CALCULATOR=$AQUA_CALCULATOR
export AQUA_POOL_ADDRESS=$AQUA_POOL_ADDRESS
export AQUA_POOL_INDEX=$AQUA_POOL_INDEX
export AQUA_LP_TOKEN=$AQUA_LP_TOKEN
export AQUA_POOL_HASH=$AQUA_POOL_HASH
export AQUA_STABLE_HASH=$AQUA_STABLE_HASH
export AQUA_TOKEN_HASH=$AQUA_TOKEN_HASH

# ---- Phoenix ----
export PHOENIX_FACTORY=$PHOENIX_FACTORY
export PHOENIX_POOL_EXT=$PHOENIX_POOL

# ---- Comet ----
export COMET_FACTORY=$COMET_FACTORY
export COMET_POOL_EXT=$COMET_POOL
export COMET_POOL_HASH=$COMET_POOL_HASH

# ---- Hoops (from deploy-testnet.sh) ----
export ROUTER=$ROUTER
export ACCOUNT_DEPLOYER=$ACCOUNT_DEPLOYER
export AQUA_ADAPTER=$AQUA_ADAPTER
export COMET_ADAPTER=$COMET_ADAPTER
export PHOENIX_ADAPTER=$PHOENIX_ADAPTER
export SOROSWAP_ADAPTER=$SOROSWAP_ADAPTER
export TEST_ACCOUNT=$TEST_ACCOUNT
EOF

cat > $OUTPUT_JSON << EOF
{
  "network": "testnet",
  "deployed_at": "$TIMESTAMP",
  "deployer": "$DEPLOYER",
  "tokens": {
    "xlm_sac": "$XLM_SAC",
    "usdc": "$USDC_TOKEN"
  },
  "amm": {
    "soroswap": {
      "factory": "$SOROSWAP_FACTORY",
      "router": "$SOROSWAP_ROUTER",
      "pair_usdc_xlm": "$SOROSWAP_PAIR"
    },
    "aquarius": {
      "router": "$AQUA_ROUTER",
      "plane": "$AQUA_PLANE",
      "calculator": "$AQUA_CALCULATOR",
      "pool_usdc_xlm": "$AQUA_POOL_ADDRESS",
      "pool_index": "$AQUA_POOL_INDEX",
      "lp_token": "$AQUA_LP_TOKEN"
    },
    "phoenix": {
      "factory": "$PHOENIX_FACTORY",
      "pool_usdc_xlm": "$PHOENIX_POOL"
    },
    "comet": {
      "factory": "$COMET_FACTORY",
      "pool_usdc_xlm": "$COMET_POOL"
    }
  },
  "hoops": {
    "router": "$ROUTER",
    "account_deployer": "$ACCOUNT_DEPLOYER",
    "adapters": {
      "aqua": "$AQUA_ADAPTER",
      "comet": "$COMET_ADAPTER",
      "phoenix": "$PHOENIX_ADAPTER",
      "soroswap": "$SOROSWAP_ADAPTER"
    },
    "test_account": "$TEST_ACCOUNT"
  }
}
EOF

echo "Addresses saved to:"
echo "  $OUTPUT_FILE"
echo "  $OUTPUT_JSON"

# Generate .env.testnet for the dashboard UI
DASHBOARD_ENV="../hoops_dashboard-ui/.env.testnet"
if [ -d "../hoops_dashboard-ui" ]; then
cat > $DASHBOARD_ENV << EOF
# Hoops Testnet - Dashboard Environment
# Auto-generated by deploy-testnet-suite-resume.sh on $TIMESTAMP
# Usage: cp .env.testnet .env.local

# Tokens
NEXT_PUBLIC_XLM_SAC=$XLM_SAC
NEXT_PUBLIC_USDC_TOKEN=$USDC_TOKEN

# Hoops Core
NEXT_PUBLIC_ROUTER=$ROUTER
NEXT_PUBLIC_ACCOUNT_DEPLOYER=$ACCOUNT_DEPLOYER
NEXT_PUBLIC_ACCOUNT_WASM_HASH=b68c3365e1ee6f049fec334a3183a634a3e1a8cec985776819869173892f482e

# Hoops Adapters
NEXT_PUBLIC_AQUA_ADAPTER=$AQUA_ADAPTER
NEXT_PUBLIC_COMET_ADAPTER=$COMET_ADAPTER
NEXT_PUBLIC_PHOENIX_ADAPTER=$PHOENIX_ADAPTER
NEXT_PUBLIC_SOROSWAP_ADAPTER=$SOROSWAP_ADAPTER

# External AMMs
NEXT_PUBLIC_SOROSWAP_FACTORY=$SOROSWAP_FACTORY
NEXT_PUBLIC_SOROSWAP_ROUTER=$SOROSWAP_ROUTER
NEXT_PUBLIC_SOROSWAP_PAIR=$SOROSWAP_PAIR
NEXT_PUBLIC_AQUA_ROUTER_EXT=$AQUA_ROUTER
NEXT_PUBLIC_AQUA_POOL=$AQUA_POOL_ADDRESS
NEXT_PUBLIC_PHOENIX_FACTORY=$PHOENIX_FACTORY
NEXT_PUBLIC_PHOENIX_POOL=$PHOENIX_POOL
NEXT_PUBLIC_COMET_FACTORY=$COMET_FACTORY
NEXT_PUBLIC_COMET_POOL=$COMET_POOL
EOF
echo "  $DASHBOARD_ENV (copy to .env.local)"
fi

echo ""
echo "================================================================"
echo "  DEPLOYMENT COMPLETE"
echo "================================================================"
echo ""
echo "Tokens:"
echo "  XLM SAC:     $XLM_SAC"
echo "  USDC:        $USDC_TOKEN"
echo ""
echo "Soroswap:"
echo "  Factory:     $SOROSWAP_FACTORY"
echo "  Router:      $SOROSWAP_ROUTER"
echo "  USDC/XLM:    $SOROSWAP_PAIR"
echo ""
echo "Aquarius:"
echo "  Router:      $AQUA_ROUTER"
echo "  USDC/XLM:    $AQUA_POOL_ADDRESS"
echo ""
echo "Phoenix:"
echo "  Factory:     $PHOENIX_FACTORY"
echo "  USDC/XLM:    $PHOENIX_POOL"
echo ""
echo "Comet:"
echo "  Factory:     $COMET_FACTORY"
echo "  USDC/XLM:    $COMET_POOL"
echo ""
echo "Next steps:"
echo "  1. cd hoops_dashboard-ui && cp .env.testnet .env.local"
echo "  2. npm run dev"
echo "  3. Test deposit at /tests/contracts"
echo ""
