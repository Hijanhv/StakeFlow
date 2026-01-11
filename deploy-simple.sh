#!/usr/bin/env bash
set -e

source .env

echo "🚀 Deploying StCSPR Token Contract..."

# Create and sign the deploy
casper-client put-deploy \
  --node-address https://node.testnet.cspr.cloud/rpc \
  --chain-name casper-test \
  --secret-key keys/secret_key.pem \
  --payment-amount 400000000000 \
  --session-path stakeflow/wasm/StCSPRToken.wasm

echo "✅ StCSPR Token deployed!"
