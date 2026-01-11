# StakeFlow Testnet Deployment Guide

**Date:** January 12, 2026  
**Status:** Ready to deploy (pending RPC access)

---

## 📋 Pre-Deployment Checklist

### ✅ Completed
- [x] Smart contracts compiled (7 WASM files)
- [x] All tests passing (27/27)
- [x] Keys generated
- [x] Account created on testnet

### 🔑 Account Information
- **Public Key (Hex):** `01f483bce2fdecda2c43a5924179d82f9490f0147ab20d3b2c0ddc8662328c3333`
- **Account Explorer:** https://testnet.cspr.live/account/01f483bce2fdecda2c43a5924179d82f9490f0147ab20d3b2c0ddc8662328c3333
- **Keys Location:** `/Users/janhv/Desktop/StakeFlow/keys/`

---

## 🚀 Deployment Steps

### Step 1: Get Testnet CSPR Tokens

#### Option A: Faucet Request
Visit the testnet faucet and request tokens:
```bash
# Open faucet in browser
open https://testnet.cspr.live/tools/faucet

# Paste your public key:
01f483bce2fdecda2c43a5924179d82f9490f0147ab20d3b2c0ddc8662328c3333
```

#### Option B: Request via Casper Discord
1. Join: https://discord.gg/caspernetwork
2. Find `#testnet-faucet` channel
3. Request tokens with your public key

**Recommended amount:** 1000 CSPR (contracts are large, need ~500 CSPR each)

---

### Step 2: Get CSPR.cloud API Access

CSPR.cloud provides reliable testnet RPC access:

1. **Sign up:** https://console.cspr.build/sign-up
2. **Get API Token:** Copy your access token from the dashboard
3. **Test connection:**
```bash
curl -X POST "https://node.testnet.cspr.cloud/rpc" \
  -H "Content-Type: application/json" \
  -H "Authorization: YOUR_API_TOKEN" \
  -d '{"jsonrpc":"2.0","id":1,"method":"info_get_status"}'
```

---

### Step 3: Deploy Contracts

Once you have tokens and RPC access, deploy the contracts:

#### Deploy StakeFlowVaultV3 (Main Contract)
```bash
cd /Users/janhv/Desktop/StakeFlow

casper-client put-deploy \
  --node-address https://node.testnet.cspr.cloud/rpc \
  --chain-name casper-test \
  --secret-key keys/secret_key.pem \
  --payment-amount 500000000000 \
  --session-path stakeflow/wasm/StakeFlowVaultV3.wasm

# Note: If CSPR.cloud requires auth header, you may need to use curl:
# See alternative deployment method below
```

#### Alternative: Deploy via curl (if client doesn't support headers)
```bash
# Build the deployment JSON
DEPLOY_HASH=$(casper-client make-deploy \
  --chain-name casper-test \
  --secret-key keys/secret_key.pem \
  --payment-amount 500000000000 \
  --session-path stakeflow/wasm/StakeFlowVaultV3.wasm \
  --output json)

# Send via curl with auth header
curl -X POST "https://node.testnet.cspr.cloud/rpc" \
  -H "Content-Type: application/json" \
  -H "Authorization: YOUR_API_TOKEN" \
  -d "$DEPLOY_HASH"
```

#### Deploy StCSPRToken
```bash
casper-client put-deploy \
  --node-address https://node.testnet.cspr.cloud/rpc \
  --chain-name casper-test \
  --secret-key keys/secret_key.pem \
  --payment-amount 400000000000 \
  --session-path stakeflow/wasm/StCSPRToken.wasm
```

#### Deploy Governance Contract
```bash
casper-client put-deploy \
  --node-address https://node.testnet.cspr.cloud/rpc \
  --chain-name casper-test \
  --secret-key keys/secret_key.pem \
  --payment-amount 400000000000 \
  --session-path stakeflow/wasm/StakeFlowGovernance.wasm
```

---

### Step 4: Verify Deployment

#### Check Deploy Status
```bash
# Get the deploy hash from the output above, then:
casper-client get-deploy \
  --node-address https://node.testnet.cspr.cloud/rpc \
  DEPLOY_HASH

# Or check in explorer:
open "https://testnet.cspr.live/deploy/DEPLOY_HASH"
```

#### Get Contract Hash
```bash
# After successful deployment, get the contract package hash:
casper-client query-global-state \
  --node-address https://node.testnet.cspr.cloud/rpc \
  --state-root-hash STATE_ROOT_HASH \
  --key account-hash-YOUR_ACCOUNT_HASH
```

---

## 📦 Available Contracts

All contracts are compiled and ready in `stakeflow/wasm/`:

| Contract | Size | Description |
|----------|------|-------------|
| `StakeFlowVaultV3.wasm` | 384KB | Main staking vault (latest version) |
| `StakeFlowGovernance.wasm` | 352KB | DAO governance system |
| `StCSPRToken.wasm` | 341KB | Liquid staking token (CEP-18) |
| `StakeFlowMinimal.wasm` | 297KB | Minimal demo version |
| `StakeFlowVaultV2.wasm` | 378KB | Previous vault version |
| `StakeFlowVault.wasm` | 371KB | Original vault version |
| `Flipper.wasm` | 156KB | Test contract |

**Recommended deployment order:**
1. `StCSPRToken.wasm` first (other contracts depend on it)
2. `StakeFlowVaultV3.wasm` (main functionality)
3. `StakeFlowGovernance.wasm` (optional, for DAO features)

---

## 🛠️ Alternative: Local Testing

If testnet deployment is blocked, you can demonstrate functionality locally:

### Using Odra Test Framework
```bash
cd /Users/janhv/Desktop/StakeFlow/stakeflow

# Run all tests
cargo odra test

# Run specific test with output
cargo test test_deposit_and_mint -- --nocapture
cargo test test_withdrawal_queue -- --nocapture
cargo test test_governance_voting -- --nocapture
```

### Using NCTL (Local Casper Network)
```bash
# Pull NCTL Docker image
docker pull makesoftware/casper-nctl

# Start local testnet
docker run -d -p 11101:11101 -p 14101:14101 \
  --name casper-nctl makesoftware/casper-nctl

# Deploy to local network (no tokens needed)
casper-client put-deploy \
  --node-address http://localhost:11101 \
  --chain-name casper-net-1 \
  --secret-key keys/secret_key.pem \
  --payment-amount 500000000000 \
  --session-path stakeflow/wasm/StakeFlowVaultV3.wasm
```

---

## 🔧 Troubleshooting

### Issue: Insufficient funds
**Solution:** Request more CSPR from faucet or reduce payment amount (minimum ~300 CSPR per contract)

### Issue: RPC timeout
**Solution:** Use CSPR.cloud with API token instead of public endpoints

### Issue: Contract execution failed
**Check:** 
- Contract initialization parameters
- Sufficient payment amount
- Chain name is correct (`casper-test` for testnet)

### Issue: Can't connect to testnet
**Status:** Public RPC endpoints may be down
**Solution:** Use CSPR.cloud (requires free registration)

---

## 📊 Deployment History

### Attempt #1
- **Date:** Previous
- **Deploy Hash:** `9f49ce41cad9bd18fdcdaca958ef003c672d6d2b513c9fa87f98ca7f7c9d1c0a`
- **Result:** Error (User error: 64658)
- **Cost:** 250 CSPR

### Attempt #2
- **Date:** Previous
- **Deploy Hash:** `225a60a35873fae41adacdbd66bacf48e42513b0d9c38ac05cbd899f285ea3bc`
- **Result:** Insufficient funds
- **Cost:** Attempted 400 CSPR

---

## 📞 Support Resources

- **Casper Discord:** https://discord.gg/caspernetwork
- **Developer Telegram:** https://t.me/CSPRDevelopers
- **Documentation:** https://docs.casper.network
- **CSPR.cloud Docs:** https://docs.cspr.cloud

---

## ✅ Next Steps

1. ⏳ **Request testnet CSPR** from faucet (need ~1000 CSPR)
2. 🔑 **Sign up for CSPR.cloud** to get reliable RPC access
3. 🚀 **Deploy contracts** using the commands above
4. ✨ **Verify on explorer** at testnet.cspr.live

---

**Ready to deploy when:**
- Account balance ≥ 1000 CSPR ✓ (pending faucet)
- RPC endpoint accessible ✓ (CSPR.cloud available)
- Contracts compiled ✅
- Keys available ✅
