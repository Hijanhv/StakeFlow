#!/bin/bash

# StakeFlow Testnet Deployment with Initialization
# This script deploys contracts and calls their init functions

set -e

source .env

echo "🚀 StakeFlow Testnet Deployment (With Initialization)"
echo "======================================================"
echo ""

# Account info
ACCOUNT_HEX="01f483bce2fdecda2c43a5924179d82f9490f0147ab20d3b2c0ddc8662328c3333"
SECRET_KEY="keys/secret_key.pem"
CHAIN_NAME="casper-test"
RPC_URL="https://node.testnet.cspr.cloud/rpc"
WASM_DIR="stakeflow/wasm"

echo "📋 Deployment Configuration:"
echo "  Account: $ACCOUNT_HEX"
echo "  Network: $CHAIN_NAME"
echo "  RPC: $RPC_URL"
echo ""

# Deploy StCSPR Token (standalone, init is called automatically by Odra)
echo "1️⃣  Deploying StCSPR Token..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

TOKEN_DEPLOY=$(casper-client make-deploy \
  --chain-name "$CHAIN_NAME" \
  --secret-key "$SECRET_KEY" \
  --payment-amount 400000000000 \
  --session-path "$WASM_DIR/StCSPRToken.wasm" \
  --ttl "30m" \
  -o token_deploy.json)

# Submit with authorization
TOKEN_DEPLOY_HASH=$(curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -H "Authorization: ${CSPR_CLOUD_API_KEY}" \
  -d @token_deploy.json | python3 -c "import sys, json; print(json.load(sys.stdin)['result']['deploy_hash'])")

rm -f token_deploy.json

echo "   ✅ Token deploy submitted: $TOKEN_DEPLOY_HASH"
echo "   ⏳ Waiting 60s for execution..."
sleep 60
echo ""

# Get token contract hash
echo "   📝 Fetching token contract hash..."
TOKEN_CONTRACT_HASH=$(curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -H "Authorization: ${CSPR_CLOUD_API_KEY}" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"id\": 1,
    \"method\": \"info_get_deploy\",
    \"params\": {
      \"deploy_hash\": \"$TOKEN_DEPLOY_HASH\"
    }
  }" | python3 -c "
import sys, json
data = json.load(sys.stdin)
effects = data.get('result', {}).get('execution_info', {}).get('execution_result', {}).get('Version2', {}).get('effects', [])
for effect in effects:
    if 'WriteContract' in effect.get('kind', {}) or 'AddContract' in effect.get('kind', {}):
        key = effect.get('key', '')
        if key.startswith('hash-'):
            print(key.replace('hash-', ''))
            break
")

if [ -z "$TOKEN_CONTRACT_HASH" ]; then
    echo "   ❌ Failed to get token contract hash"
    echo "   Check deployment at: https://testnet.cspr.live/deploy/$TOKEN_DEPLOY_HASH"
    exit 1
fi

echo "   ✅ Token Contract Hash: $TOKEN_CONTRACT_HASH"
echo ""

# Deploy Vault V3 with initialization parameters
echo "2️⃣  Deploying StakeFlow Vault V3..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Treasury = deployer account for now
TREASURY_ADDR="account-hash-${ACCOUNT_HEX:2}"
UNBONDING_DAYS="7"

VAULT_DEPLOY=$(casper-client make-deploy \
  --chain-name "$CHAIN_NAME" \
  --secret-key "$SECRET_KEY" \
  --payment-amount 400000000000 \
  --session-path "$WASM_DIR/StakeFlowVaultV3.wasm" \
  --session-arg "treasury_address:key='$TREASURY_ADDR'" \
  --session-arg "unbonding_days:u64='$UNBONDING_DAYS'" \
  --ttl "30m" \
  -o vault_deploy.json)

VAULT_DEPLOY_HASH=$(curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -H "Authorization: ${CSPR_CLOUD_API_KEY}" \
  -d @vault_deploy.json | python3 -c "import sys, json; print(json.load(sys.stdin)['result']['deploy_hash'])")

rm -f vault_deploy.json

echo "   ✅ Vault deploy submitted: $VAULT_DEPLOY_HASH"
echo "   ⏳ Waiting 60s for execution..."
sleep 60
echo ""

# Get vault contract hash
echo "   📝 Fetching vault contract hash..."
VAULT_CONTRACT_HASH=$(curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -H "Authorization: ${CSPR_CLOUD_API_KEY}" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"id\": 1,
    \"method\": \"info_get_deploy\",
    \"params\": {
      \"deploy_hash\": \"$VAULT_DEPLOY_HASH\"
    }
  }" | python3 -c "
import sys, json
data = json.load(sys.stdin)
effects = data.get('result', {}).get('execution_info', {}).get('execution_result', {}).get('Version2', {}).get('effects', [])
for effect in effects:
    if 'WriteContract' in effect.get('kind', {}) or 'AddContract' in effect.get('kind', {}):
        key = effect.get('key', '')
        if key.startswith('hash-'):
            print(key.replace('hash-', ''))
            break
")

if [ -z "$VAULT_CONTRACT_HASH" ]; then
    echo "   ❌ Failed to get vault contract hash"
    echo "   Check deployment at: https://testnet.cspr.live/deploy/$VAULT_DEPLOY_HASH"
    exit 1
fi

echo "   ✅ Vault Contract Hash: $VAULT_CONTRACT_HASH"
echo ""

# Deploy Governance with vault address
echo "3️⃣  Deploying StakeFlow Governance..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

VAULT_KEY="hash-$VAULT_CONTRACT_HASH"

GOV_DEPLOY=$(casper-client make-deploy \
  --chain-name "$CHAIN_NAME" \
  --secret-key "$SECRET_KEY" \
  --payment-amount 400000000000 \
  --session-path "$WASM_DIR/StakeFlowGovernance.wasm" \
  --session-arg "vault_address:key='$VAULT_KEY'" \
  --ttl "30m" \
  -o gov_deploy.json)

GOV_DEPLOY_HASH=$(curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -H "Authorization: ${CSPR_CLOUD_API_KEY}" \
  -d @gov_deploy.json | python3 -c "import sys, json; print(json.load(sys.stdin)['result']['deploy_hash'])")

rm -f gov_deploy.json

echo "   ✅ Governance deploy submitted: $GOV_DEPLOY_HASH"
echo "   ⏳ Waiting 60s for execution..."
sleep 60
echo ""

# Get governance contract hash
echo "   📝 Fetching governance contract hash..."
GOV_CONTRACT_HASH=$(curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -H "Authorization: ${CSPR_CLOUD_API_KEY}" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"id\": 1,
    \"method\": \"info_get_deploy\",
    \"params\": {
      \"deploy_hash\": \"$GOV_DEPLOY_HASH\"
    }
  }" | python3 -c "
import sys, json
data = json.load(sys.stdin)
effects = data.get('result', {}).get('execution_info', {}).get('execution_result', {}).get('Version2', {}).get('effects', [])
for effect in effects:
    if 'WriteContract' in effect.get('kind', {}) or 'AddContract' in effect.get('kind', {}):
        key = effect.get('key', '')
        if key.startswith('hash-'):
            print(key.replace('hash-', ''))
            break
")

if [ -z "$GOV_CONTRACT_HASH" ]; then
    echo "   ❌ Failed to get governance contract hash"
    echo "   Check deployment at: https://testnet.cspr.live/deploy/$GOV_DEPLOY_HASH"
    exit 1
fi

echo "   ✅ Governance Contract Hash: $GOV_CONTRACT_HASH"
echo ""

# Summary
echo "✨ Deployment Complete!"
echo "======================================================"
echo ""
echo "📝 Contract Addresses (add these to frontend .env.local):"
echo ""
echo "NEXT_PUBLIC_TOKEN_CONTRACT_HASH=\"$TOKEN_CONTRACT_HASH\""
echo "NEXT_PUBLIC_VAULT_CONTRACT_HASH=\"$VAULT_CONTRACT_HASH\""
echo "NEXT_PUBLIC_GOVERNANCE_CONTRACT_HASH=\"$GOV_CONTRACT_HASH\""
echo ""
echo "🔗 Testnet Explorer Links:"
echo "  Token: https://testnet.cspr.live/contract/$TOKEN_CONTRACT_HASH"
echo "  Vault: https://testnet.cspr.live/contract/$VAULT_CONTRACT_HASH"
echo "  Governance: https://testnet.cspr.live/contract/$GOV_CONTRACT_HASH"
echo ""
echo "🎉 All contracts deployed successfully with initialization!"
