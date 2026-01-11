#!/usr/bin/env bash
set -e

source .env

echo "🔍 Checking contract deployment status..."
echo ""

TOKEN_HASH="7fbc541ba8fc23e6b9daf814e9c6b34eaf513e51c7366db235f367b73b286501"
VAULT_HASH="3dd110491ad7ac672059d765fcbff603aa12cf2c1eefab833ea1b53086bc1d97"
GOV_HASH="c953f3a1bdbaacfead32f04ee668261a7e9250b8d4b4e7ddeaf8d9f4ca7ca7d3"

check_and_extract() {
  local deploy_hash=$1
  local name=$2
  
  result=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -H "Authorization: ${CSPR_CLOUD_API_KEY}" \
    "${TESTNET_RPC_URL}" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"info_get_deploy\",\"params\":{\"deploy_hash\":\"$deploy_hash\"}}")
  
  status=$(echo "$result" | python3 -c "import sys, json; data=json.load(sys.stdin); result=data.get('result',{}); exec_results=result.get('execution_results',[]); print('success' if exec_results and 'Success' in exec_results[0]['result'] else ('failed' if exec_results else 'pending'))" 2>&1)
  
  if [ "$status" = "success" ]; then
    contract_hash=$(echo "$result" | python3 -c "
import sys, json
data = json.load(sys.stdin)
exec_result = data['result']['execution_results'][0]['result']['Success']
effects = exec_result.get('effect', {}).get('transforms', [])
for transform in effects:
    if 'WriteContract' in str(transform):
        key = transform['key']
        if key.startswith('hash-'):
            print(key.replace('hash-', ''))
            break
" 2>&1)
    echo "✅ $name: $contract_hash"
    echo "$name|$contract_hash"
  elif [ "$status" = "failed" ]; then
    echo "❌ $name: FAILED"
  else
    echo "⏳ $name: Pending..."
  fi
}

echo "📝 Token Contract:"
TOKEN_CONTRACT=$(check_and_extract "$TOKEN_HASH" "Token")
echo ""

echo "📝 Vault Contract:"
VAULT_CONTRACT=$(check_and_extract "$VAULT_HASH" "Vault")
echo ""

echo "📝 Governance Contract:"
GOV_CONTRACT=$(check_and_extract "$GOV_HASH" "Governance")
echo ""

# If all succeeded, show contract hashes
if [[ "$TOKEN_CONTRACT" == *"|"* ]] && [[ "$VAULT_CONTRACT" == *"|"* ]] && [[ "$GOV_CONTRACT" == *"|"* ]]; then
  echo ""
  echo "🎉 All contracts deployed successfully!"
  echo ""
  echo "📋 Add these to stakeflow-frontend/.env.local:"
  echo "NEXT_PUBLIC_TOKEN_CONTRACT_HASH=$(echo $TOKEN_CONTRACT | cut -d'|' -f2)"
  echo "NEXT_PUBLIC_VAULT_CONTRACT_HASH=$(echo $VAULT_CONTRACT | cut -d'|' -f2)"
  echo "NEXT_PUBLIC_GOVERNANCE_CONTRACT_HASH=$(echo $GOV_CONTRACT | cut -d'|' -f2)"
fi
