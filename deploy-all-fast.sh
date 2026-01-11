#!/bin/bash
set -e
source .env

deploy() {
  casper-client make-deploy --chain-name casper-test --secret-key keys/secret_key.pem --payment-amount 400000000000 --session-path "stakeflow/wasm/$1.wasm" --ttl 30m -o d.json 2>&1 > /dev/null
  h=$(python3 -c "import json; print(json.load(open('d.json'))['hash'])")
  cat > r.json <<EOF
{"jsonrpc":"2.0","id":1,"method":"account_put_deploy","params":{"deploy":$(cat d.json)}}
EOF
  curl -s -X POST https://node.testnet.cspr.cloud/rpc -H "Content-Type: application/json" -H "Authorization: ${CSPR_CLOUD_API_KEY}" -d @r.json > /dev/null
  echo "$h"
  rm -f d.json r.json
}

echo "🚀 Deploying contracts..."
T=$(deploy "StCSPRToken")
echo "Token: $T"
sleep 60
V=$(deploy "StakeFlowVaultV3")
echo "Vault: $V"
sleep 60
G=$(deploy "StakeFlowGovernance")
echo "Gov: $G"
sleep 90

echo ""
echo "NEXT_PUBLIC_TOKEN_CONTRACT_HASH=\"$T\""
echo "NEXT_PUBLIC_VAULT_CONTRACT_HASH=\"$V\""
echo "NEXT_PUBLIC_GOVERNANCE_CONTRACT_HASH=\"$G\""
