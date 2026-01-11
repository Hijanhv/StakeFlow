#!/bin/bash

# StakeFlow V3 Deployment Script for Casper Testnet
# Deploy the new production contracts (VaultV3 + Governance + Minimal)

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}StakeFlow V3 Testnet Deployment${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Configuration
NODE_ADDRESS="https://rpc.testnet.casperlabs.io"
CHAIN_NAME="casper-test"
SECRET_KEY_PATH="./keys/secret_key.pem"

# Check if secret key exists
if [ ! -f "$SECRET_KEY_PATH" ]; then
    echo -e "${RED}Error: Secret key not found at $SECRET_KEY_PATH${NC}"
    echo "Please create keys directory and add your secret_key.pem"
    exit 1
fi

# Get public key
PUBLIC_KEY=$(casper-client account-address --public-key-path "$SECRET_KEY_PATH" 2>/dev/null || echo "error")
if [ "$PUBLIC_KEY" == "error" ]; then
    echo -e "${RED}Error: Could not read public key${NC}"
    exit 1
fi

echo -e "${YELLOW}Account Public Key:${NC} $PUBLIC_KEY"
echo ""

# Check account balance
echo -e "${YELLOW}Checking account balance...${NC}"
STATE_ROOT=$(casper-client get-state-root-hash --node-address "$NODE_ADDRESS" 2>/dev/null | jq -r '.result.state_root_hash' || echo "")

if [ -z "$STATE_ROOT" ] || [ "$STATE_ROOT" == "null" ]; then
    echo -e "${RED}Warning: Could not connect to testnet RPC${NC}"
    echo "Proceeding anyway - deployment will fail if insufficient funds"
else
    echo -e "${GREEN}Connected to testnet successfully${NC}"
fi

echo ""
echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}Deployment Options:${NC}"
echo -e "${YELLOW}========================================${NC}"
echo "1. Deploy StakeFlowMinimal (297KB) - ~300 CSPR"
echo "2. Deploy StakeFlowVaultV3 (384KB) - ~400 CSPR"
echo "3. Deploy StakeFlowGovernance (352KB) - ~350 CSPR"
echo "4. Deploy ALL contracts sequentially"
echo "5. Exit"
echo ""
read -p "Select option (1-5): " choice

deploy_contract() {
    local CONTRACT_NAME=$1
    local WASM_PATH=$2
    local PAYMENT_AMOUNT=$3

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Deploying $CONTRACT_NAME${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo -e "${YELLOW}WASM Path:${NC} $WASM_PATH"
    echo -e "${YELLOW}Payment:${NC} $PAYMENT_AMOUNT CSPR"

    # Check if WASM file exists
    if [ ! -f "$WASM_PATH" ]; then
        echo -e "${RED}Error: WASM file not found at $WASM_PATH${NC}"
        return 1
    fi

    # Get file size
    FILE_SIZE=$(ls -lh "$WASM_PATH" | awk '{print $5}')
    echo -e "${YELLOW}Contract Size:${NC} $FILE_SIZE"
    echo ""

    # Deploy
    echo -e "${YELLOW}Deploying to testnet...${NC}"

    DEPLOY_HASH=$(casper-client put-deploy \
        --node-address "$NODE_ADDRESS" \
        --chain-name "$CHAIN_NAME" \
        --secret-key "$SECRET_KEY_PATH" \
        --payment-amount "${PAYMENT_AMOUNT}000000000" \
        --session-path "$WASM_PATH" \
        2>&1 | grep -oP 'deploy_hash.*:\s*"\K[^"]+' || echo "")

    if [ -z "$DEPLOY_HASH" ]; then
        echo -e "${RED}Deployment failed or hash not found${NC}"
        echo "Check RPC connection and account balance"
        return 1
    fi

    echo -e "${GREEN}✅ Deployment submitted!${NC}"
    echo -e "${YELLOW}Deploy Hash:${NC} $DEPLOY_HASH"
    echo -e "${YELLOW}Explorer:${NC} https://testnet.cspr.live/deploy/$DEPLOY_HASH"
    echo ""
    echo "Waiting 30 seconds for deployment to process..."
    sleep 30

    # Check deployment status
    echo -e "${YELLOW}Checking deployment status...${NC}"
    casper-client get-deploy \
        --node-address "$NODE_ADDRESS" \
        "$DEPLOY_HASH" 2>&1 | head -50

    echo ""
    echo -e "${GREEN}$CONTRACT_NAME deployment complete!${NC}"
    echo -e "${YELLOW}Save this deploy hash:${NC} $DEPLOY_HASH"
    echo ""

    return 0
}

case $choice in
    1)
        deploy_contract "StakeFlowMinimal" "wasm/StakeFlowMinimal.wasm" "300"
        ;;
    2)
        # VaultV3 requires initialization args
        echo ""
        echo -e "${YELLOW}VaultV3 requires initialization parameters:${NC}"
        read -p "Enter treasury address (or press Enter for deployer): " TREASURY
        if [ -z "$TREASURY" ]; then
            TREASURY=$PUBLIC_KEY
        fi
        read -p "Enter unbonding period in days (default 7): " UNBONDING_DAYS
        if [ -z "$UNBONDING_DAYS" ]; then
            UNBONDING_DAYS=7
        fi

        echo -e "${YELLOW}Treasury:${NC} $TREASURY"
        echo -e "${YELLOW}Unbonding Period:${NC} $UNBONDING_DAYS days"
        echo ""
        echo -e "${RED}Note: This deployment requires runtime args which need to be added to the command${NC}"
        echo "For now, deploying without args (will need manual initialization)"
        deploy_contract "StakeFlowVaultV3" "wasm/StakeFlowVaultV3.wasm" "400"
        ;;
    3)
        # Governance requires vault address
        echo ""
        echo -e "${YELLOW}Governance requires the VaultV3 contract address${NC}"
        read -p "Enter VaultV3 contract hash: " VAULT_HASH
        if [ -z "$VAULT_HASH" ]; then
            echo -e "${RED}Error: Vault hash required${NC}"
            exit 1
        fi

        echo -e "${RED}Note: This deployment requires runtime args which need to be added to the command${NC}"
        echo "For now, deploying without args (will need manual initialization)"
        deploy_contract "StakeFlowGovernance" "wasm/StakeFlowGovernance.wasm" "350"
        ;;
    4)
        echo ""
        echo -e "${YELLOW}Deploying ALL contracts...${NC}"
        echo -e "${YELLOW}This requires ~1050 CSPR total${NC}"
        read -p "Continue? (y/n): " confirm
        if [ "$confirm" != "y" ]; then
            echo "Deployment cancelled"
            exit 0
        fi

        deploy_contract "StakeFlowMinimal" "wasm/StakeFlowMinimal.wasm" "300"
        deploy_contract "StakeFlowVaultV3" "wasm/StakeFlowVaultV3.wasm" "400"
        deploy_contract "StakeFlowGovernance" "wasm/StakeFlowGovernance.wasm" "350"

        echo ""
        echo -e "${GREEN}========================================${NC}"
        echo -e "${GREEN}All Deployments Complete!${NC}"
        echo -e "${GREEN}========================================${NC}"
        ;;
    5)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo -e "${RED}Invalid option${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Deployment Script Complete${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo "1. Verify deployment on https://testnet.cspr.live"
echo "2. Save all deploy hashes in DEPLOYMENT_STATUS.md"
echo "3. Initialize contracts with proper parameters"
echo "4. Update README.md with contract addresses"
echo ""
