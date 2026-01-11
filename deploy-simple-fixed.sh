#!/bin/bash

# Simple StakeFlow Testnet Deployment - Deploy All 3 Contracts
# Deploys Token, Vault, and Governance without complex initialization
# They will use default/self-initialization from Odra framework

set -e

source .env

echo "🚀 StakeFlow Testnet Deployment (Simplified)"
echo "=============================================="
echo ""

# Configuration
ACCOUNT_HEX="01f483bce2fdecda2c43a5924179d82f9490f0147ab20d3b2c0ddc8662328c3333"
SECRET_KEY="keys/secret_key.pem"
CHAIN_NAME="casper-test"
RPC_URL="https://node.testnet.cspr.cloud/rpc"
WASM_DIR="stakeflow/wasm"

# Function to deploy a contract
deploy_contract() {
    local name=$1
    local wasm_file=$2
    local gas=$3
    
    echo "📦 Deploying $name..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Create deploy
    casper-client make-deploy \
      --chain-name "$CHAIN_NAME" \
      --secret-key "$SECRET_KEY" \
      --payment-amount "$gas" \
      --session-path "$wasm_file" \
      --ttl "30m" \
      -o deploy_temp.json > /dev/null 2>&1
    
    # Extract deploy hash
    local deploy_hash=$(python3 -c "import json; print(json.load(open('deploy_temp.json'))['hash'])")
    
    # Wrap in JSON-RPC request
    cat > rpc_request.json <<EOF
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "account_put_deploy",
  "params": {
    "deploy": $(cat deploy_temp.json)
  }
}
EOF
    
    # Submit
    local response=$(curl -s -X POST "$RPC_URL" \
      -H "Content-Type: application/json" \
      -H "Authorization: ${CSPR_CLOUD_API_KEY}" \
      -d @rpc_request.json)
    
    # Clean up
    rm -f deploy_temp.json rpc_request.json
    
    # Check response
    if echo "$response" | grep -q "error"; then
        echo "   ❌ Deployment failed:"
        echo "$response" | python3 -m json.tool
        return 1
    fi
    
    echo "   ✅ Deploy Hash: $deploy_hash"
    echo "   🔗 https://testnet.cspr.live/deploy/$deploy_hash"
    echo ""
    
    # Store hash for later
    echo "$deploy_hash"
}

# Deploy contracts
echo "Starting deployment sequence..."
echo ""

TOKEN_HASH=$(deploy_contract "StCSPR Token" "$WASM_DIR/StCSPRToken.wasm" "400000000000")
echo "⏳ Waiting 90s for token to execute..."
sleep 90

VAULT_HASH=$(deploy_contract "StakeFlow Vault V3" "$WASM_DIR/StakeFlowVaultV3.wasm" "400000000000")
echo "⏳ Waiting 90s for vault to execute..."
sleep 90

GOV_HASH=$(deploy_contract "StakeFlow Governance" "$WASM_DIR/StakeFlowGovernance.wasm" "400000000000")
echo "⏳ Waiting 90s for governance to execute..."
sleep 90

echo ""
echo "✨ All Contracts Deployed!"
echo "=============================================="
echo ""
echo "📝 Deploy Hashes:"
echo "  Token:      $TOKEN_HASH"
echo "  Vault:      $VAULT_HASH"
echo "  Governance: $GOV_HASH"
echo ""
echo "⏳ Now checking for contract hashes..."
echo ""

# Run the check-deployments script
./check-deployments.sh

echo ""
echo "🎉 Deployment Complete!"
