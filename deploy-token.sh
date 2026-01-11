#!/usr/bin/env bash
set -e

source .env

echo "🚀 Deploying StCSPR Token Contract..."

# Remove old json file if exists
rm -f json

# Create the deploy
echo "Creating deploy..."
casper-client make-deploy \
  --chain-name casper-test \
  --secret-key keys/secret_key.pem \
  --payment-amount 400000000000 \
  --session-path stakeflow/wasm/StCSPRToken.wasm \
  --output json

echo "Deploy created!"

# Read the deploy JSON
DEPLOY=$(cat json)

# Send the deploy using curl with authentication
echo "Sending deploy to network..."
RESULT=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -H "Authorization: ${CSPR_CLOUD_API_KEY}" \
  "${TESTNET_RPC_URL}" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "account_put_deploy",
    "params": {
      "deploy": '"${DEPLOY}"'
    }
  }')

# Extract deploy hash
DEPLOY_HASH=$(echo "$RESULT" | python3 -c "import sys, json; print(json.load(sys.stdin)['result']['deploy_hash'])" 2>/dev/null || echo "ERROR")

if [ "$DEPLOY_HASH" = "ERROR" ]; then
  echo "❌ Deploy failed!"
  echo "$RESULT" | python3 -m json.tool
  exit 1
fi

echo "✅ Deploy submitted successfully!"
echo "Deploy hash: ${DEPLOY_HASH}"
echo "Check status at: https://testnet.cspr.live/deploy/${DEPLOY_HASH}"

# Clean up
rm -f json
