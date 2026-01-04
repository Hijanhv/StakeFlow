#!/bin/bash

# StakeFlow Deployment Script
echo "🚀 Deploying StakeFlow to Casper Testnet..."

# Deploy using put-transaction (new method)
casper-client put-transaction \
  --node-address https://rpc.testnet.casperlabs.io \
  --chain-name casper-test \
  --secret-key ../keys/secret_key.pem \
  --payment-amount 200000000000 \
  --session-path wasm/StakeFlowVault.wasm

echo "✅ Deployment transaction sent!"
echo "Check status at: https://testnet.cspr.live"
