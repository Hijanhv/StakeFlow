#!/usr/bin/env bash
set -e

source .env

echo "🚀 Deploying StakeFlow Vault V3 Contract..."

rm -f vault_deploy.json

casper-client make-deploy \
  --chain-name casper-test \
  --secret-key keys/secret_key.pem \
  --payment-amount 500000000000 \
  --session-path stakeflow/wasm/StakeFlowVaultV3.wasm \
  --output vault_deploy.json

DEPLOY=$(cat vault_deploy.json)

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

DEPLOY_HASH=$(echo "$RESULT" | python3 -c "import sys, json; print(json.load(sys.stdin)['result']['deploy_hash'])" 2>/dev/null || echo "ERROR")

if [ "$DEPLOY_HASH" = "ERROR" ]; then
  echo "❌ Deploy failed!"
  echo "$RESULT" | python3 -m json.tool
  exit 1
fi

echo "✅ Vault deployed!"
echo "Deploy hash: ${DEPLOY_HASH}"
echo "Check at: https://testnet.cspr.live/deploy/${DEPLOY_HASH}"

rm -f vault_deploy.json
