#!/bin/bash

# StakeFlow Testnet Deployment Script
# Uses CSPR.cloud API for reliable testnet access

set -e

# Load environment variables
source .env

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   StakeFlow Testnet Deployment Script    ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""

# Function to check balance
check_balance() {
    echo -e "${YELLOW}📊 Checking account balance...${NC}"
    
    # Get state root hash
    STATE_ROOT=$(curl -s "$TESTNET_RPC_URL" \
        -H "Content-Type: application/json" \
        -H "Authorization: $CSPR_CLOUD_API_KEY" \
        -d '{"jsonrpc":"2.0","id":1,"method":"chain_get_state_root_hash"}' \
        | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['state_root_hash'])")
    
    echo "State Root Hash: $STATE_ROOT"
    
    # Get account info
    ACCOUNT_INFO=$(curl -s "$TESTNET_RPC_URL" \
        -H "Content-Type: application/json" \
        -H "Authorization: $CSPR_CLOUD_API_KEY" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"query_global_state\",\"params\":{\"state_identifier\":{\"StateRootHash\":\"$STATE_ROOT\"},\"key\":\"$ACCOUNT_HASH\",\"path\":[]}}")
    
    echo "$ACCOUNT_INFO" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    if 'result' in data and 'stored_value' in data['result']:
        purse = data['result']['stored_value']['Account']['main_purse']
        print(f'✅ Account found')
        print(f'Main Purse: {purse}')
    else:
        print('❌ Account not found or empty')
        sys.exit(1)
except Exception as e:
    print(f'❌ Error: {e}')
    sys.exit(1)
"
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Account has 0 balance. Please request testnet CSPR first:${NC}"
        echo -e "${YELLOW}Visit: https://testnet.cspr.live/tools/faucet${NC}"
        echo -e "${YELLOW}Public Key: $PUBLIC_KEY_HEX${NC}"
        exit 1
    fi
}

# Function to deploy a contract
deploy_contract() {
    local CONTRACT_PATH=$1
    local CONTRACT_NAME=$2
    local PAYMENT_AMOUNT=$3
    
    echo ""
    echo -e "${GREEN}🚀 Deploying $CONTRACT_NAME...${NC}"
    echo -e "Contract: $CONTRACT_PATH"
    echo -e "Payment: $PAYMENT_AMOUNT motes"
    
    # Create the deploy (without sending yet)
    DEPLOY_JSON=$(casper-client make-deploy \
        --chain-name casper-test \
        --secret-key keys/secret_key.pem \
        --payment-amount "$PAYMENT_AMOUNT" \
        --session-path "$CONTRACT_PATH" \
        --output json 2>&1)
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Failed to create deploy${NC}"
        echo "$DEPLOY_JSON"
        return 1
    fi
    
    # Send the deploy via curl with auth header
    DEPLOY_RESULT=$(echo "$DEPLOY_JSON" | curl -s "$TESTNET_RPC_URL" \
        -H "Content-Type: application/json" \
        -H "Authorization: $CSPR_CLOUD_API_KEY" \
        -d @-)
    
    # Extract deploy hash
    DEPLOY_HASH=$(echo "$DEPLOY_RESULT" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    if 'result' in data:
        print(data['result']['deploy_hash'])
    else:
        print('ERROR: ' + json.dumps(data))
        sys.exit(1)
except Exception as e:
    print(f'ERROR: {e}')
    sys.exit(1)
")
    
    if [[ "$DEPLOY_HASH" == ERROR* ]]; then
        echo -e "${RED}❌ Deployment failed${NC}"
        echo "$DEPLOY_HASH"
        return 1
    fi
    
    echo -e "${GREEN}✅ Deploy submitted!${NC}"
    echo -e "Deploy Hash: ${GREEN}$DEPLOY_HASH${NC}"
    echo -e "Explorer: https://testnet.cspr.live/deploy/$DEPLOY_HASH"
    echo -e "${YELLOW}⏳ Waiting for deployment to complete...${NC}"
    
    # Wait and check status
    sleep 5
    
    for i in {1..12}; do
        sleep 10
        STATUS=$(curl -s "$TESTNET_RPC_URL" \
            -H "Content-Type: application/json" \
            -H "Authorization: $CSPR_CLOUD_API_KEY" \
            -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"info_get_deploy\",\"params\":{\"deploy_hash\":\"$DEPLOY_HASH\"}}" \
            | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    if 'result' in data and 'execution_results' in data['result']:
        results = data['result']['execution_results']
        if results:
            success = results[0].get('result', {}).get('Success') is not None
            print('SUCCESS' if success else 'FAILED')
        else:
            print('PENDING')
    else:
        print('PENDING')
except:
    print('PENDING')
")
        
        if [ "$STATUS" = "SUCCESS" ]; then
            echo -e "${GREEN}✅ $CONTRACT_NAME deployed successfully!${NC}"
            return 0
        elif [ "$STATUS" = "FAILED" ]; then
            echo -e "${RED}❌ $CONTRACT_NAME deployment failed${NC}"
            return 1
        fi
        
        echo -e "${YELLOW}⏳ Still pending... (${i}/12)${NC}"
    done
    
    echo -e "${YELLOW}⚠️  Deployment status unknown (check explorer)${NC}"
    return 0
}

# Main deployment flow
main() {
    echo -e "${YELLOW}🔑 Account: $PUBLIC_KEY_HEX${NC}"
    echo -e "${YELLOW}🌐 RPC: $TESTNET_RPC_URL${NC}"
    echo ""
    
    # Check balance first
    check_balance
    
    echo ""
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo -e "${GREEN}   Starting Contract Deployments${NC}"
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    
    # Deploy contracts in order
    # 1. Token contract (needed by vault)
    deploy_contract \
        "stakeflow/wasm/StCSPRToken.wasm" \
        "StCSPR Token" \
        "400000000000"
    
    # 2. Main vault contract
    deploy_contract \
        "stakeflow/wasm/StakeFlowVaultV3.wasm" \
        "StakeFlow Vault V3" \
        "500000000000"
    
    # 3. Governance contract
    deploy_contract \
        "stakeflow/wasm/StakeFlowGovernance.wasm" \
        "StakeFlow Governance" \
        "400000000000"
    
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     Deployment Complete!                 ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}📊 View your deployments:${NC}"
    echo -e "https://testnet.cspr.live/account/$PUBLIC_KEY_HEX"
}

# Run main function
main
