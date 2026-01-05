# StakeFlow Enhanced Contract Deployment Guide

## 🎉 Latest Deployment (Day 3 - January 5, 2026)

### Deployed Contract Information
- **Deploy Hash:** `9f49ce41cad9bd18fdcdaca958ef003c672d6d2b513c9fa87f98ca7f7c9d1c0a`
- **Explorer Link:** https://testnet.cspr.live/deploy/9f49ce41cad9bd18fdcdaca958ef003c672d6d2b513c9fa87f98ca7f7c9d1c0a
- **Network:** Casper Testnet
- **Block Height:** 6449197
- **Contract Size:** 375KB (Enhanced with auto-rebalancing & cross-chain features)
- **Gas Used:** 1,152,435
- **Payment Amount:** 250 CSPR

### Features Deployed
✅ Liquid staking with multi-validator support
✅ Auto-rebalancing based on performance
✅ Validator management & scoring
✅ Cross-chain deposit infrastructure  
✅ Advanced portfolio analytics
✅ Risk scoring system
✅ Staking rewards & auto-compounding
✅ 10 comprehensive tests passing

## Prerequisites

- ✅ Rust and Casper CLI tools installed
- ✅ 1000+ CSPR testnet tokens in your account
- ✅ StakeFlowVault.wasm compiled (375KB)

## Account Information

**Local Account (has funds):**
- Public Key: `01f483bce2fdecda2c43a5924179d82f9490f0147ab20d3b2c0ddc8662328c3333`
- Balance: ~1000 CSPR (testnet)
- Private Key: `../keys/secret_key.pem`

## Deployment Command (Used)

```bash
casper-client put-deploy \
  --node-address https://node.testnet.casper.network/rpc \
  --chain-name casper-test \
  --secret-key ../keys/secret_key.pem \
  --payment-amount 250000000000 \
  --session-path wasm/StakeFlowVault.wasm
```

**Note:** Increased payment amount to 250 CSPR due to larger contract size (375KB vs previous 325KB)

## Alternative Testnet RPC Endpoints

If the official endpoint is down, try these alternatives:

1. **CSPR.cloud Testnet:** `https://node.testnet.cspr.cloud/rpc` (may require auth)
2. **Community Node 1:** `http://65.21.235.219:7777`
3. **Community Node 2:** `http://195.201.174.222:7777`

## Verify Deployment

After deployment, you'll receive a transaction hash. Check the status:

```bash
# Get transaction/deploy hash from output
DEPLOY_HASH=<your-deploy-hash-here>

# Check status
casper-client get-deploy \
  --node-address https://node.testnet.casper.network/rpc \
  $DEPLOY_HASH
```

Or visit: https://testnet.cspr.live and search for your deploy hash or account address.

## Common Issues

### Issue: "failed to get response for rpc-id"
**Solution:** Testnet RPC may be temporarily down. Try:
1. Wait a few minutes and retry
2. Try alternative RPC endpoints above
3. Check Casper Discord for testnet status

### Issue: "insufficient balance"
**Solution:** Request more tokens from faucet at https://testnet.cspr.live/tools/faucet

### Issue: "invalid secret key"
**Solution:** Ensure you're using the correct path: `../keys/secret_key.pem`

## Post-Deployment

Once deployed successfully:

1. **Save the Contract Hash** - You'll receive a contract package hash
2. **Update README.md** - Add the contract hash to the deployment section
3. **Test Interaction** - Try depositing/withdrawing CSPR
4. **Update GitHub** - Commit and push the contract hash

## Contract Interaction

After deployment, interact with the contract:

```bash
# Call deposit (example)
casper-client put-transaction session \
  --node-address https://node.testnet.casper.network/rpc \
  --chain-name casper-test \
  --secret-key ../keys/secret_key.pem \
  --payment-amount 5000000000 \
  --gas-price-tolerance 5 \
  --standard-payment true \
  --session-hash <CONTRACT_HASH> \
  --session-entry-point "deposit" \
  --transferred-value 100000000000
```

## Resources

- [Casper Testnet Explorer](https://testnet.cspr.live)
- [Casper RPC Documentation](https://docs.casper.network/operators/setup/node-endpoints)
- [CSPR.cloud Docs](https://docs.cspr.cloud/)
- [Casper Discord](https://discord.com/invite/caspernetwork)

---

**Note:** If testnet RPC is experiencing issues (January 2026), wait for network stability or check Casper Discord for updates on testnet status.
