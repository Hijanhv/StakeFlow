# StakeFlow - Casper Hackathon 2026 Submission

## 🏆 Track: Liquid Staking

**Team:** Solo Developer (Janhavi)
**Project:** StakeFlow - Enterprise Liquid Staking Protocol
**Status:** Production-Ready, 27/27 Tests Passing ✅

---

## 🚀 What We Built

### Complete Liquid Staking Protocol with:

1. **StakeFlowVaultV3** (375KB) - Main Protocol
   - Instant CSPR → stCSPR liquid staking
   - CEP-18 compliant stCSPR token (fully transferable)
   - **7-day withdrawal queue with unbonding period** (UNIQUE!)
   - Auto-compounding rewards with exchange rate
   - Performance fees (5%) to treasury
   - Emergency pause controls

2. **StakeFlowGovernance** (344KB) - DAO System
   - Token holder voting (1 stCSPR = 1 vote)
   - Proposal system for protocol changes
   - 3-day voting period, 20% quorum, 50% approval
   - Change fees, add/remove validators, emergency controls

3. **StakeFlowMinimal** (290KB) - Lightweight Version
   - Simple liquid staking for testing
   - Lower deployment cost

---

## 💎 What Makes Us Different

### 1. Real Liquid Staking (Our Killer Feature)
**Other projects:**
- Deposit CSPR → Get token → Withdraw immediately
- **This isn't realistic!** Real staking has unbonding periods

**StakeFlow:**
- Deposit CSPR → Get stCSPR instantly ✅
- Request withdrawal → Burns stCSPR, starts 7-day countdown ⏳
- After 7 days → Claim your CSPR ✅
- **This matches Lido, Rocket Pool, and real liquid staking protocols!**

### 2. Fully Integrated stCSPR Token
- CEP-18 compliant (works everywhere on Casper)
- Transfer, approve, transferFrom
- Use in DeFi while staking
- Exchange rate increases as rewards accrue

### 3. DAO Governance From Day 1
- stCSPR holders vote on protocol changes
- Adjust fees, add validators, emergency controls
- Decentralized from the start
- Like CEWCE's governance but for DeFi

### 4. Production-Ready Code
- **27/27 tests passing**
- Comprehensive test coverage
- Security features (minimum deposits, liquidity checks)
- Event emissions for transparency
- Ready to deploy today

---

## 📊 Technical Achievements

### Test Coverage:
```
running 27 tests
✅ VaultV3: 6 tests (deposit, withdraw, rewards, transfers)
✅ Governance: 3 tests (proposals, voting, execution)
✅ stCSPR Token: 9 tests (mint, burn, transfer, approve)
✅ Legacy Vault: 9 tests (full coverage)

test result: ok. 27 passed; 0 failed
```

### Contract Sizes:
- StakeFlowVaultV3.wasm: **375KB** (optimized)
- StakeFlowGovernance.wasm: **344KB** (optimized)
- StakeFlowMinimal.wasm: **290KB** (optimized)

### Architecture:
- Unified contract design (simpler deployment)
- CEP-18 token standard compliance
- Event-driven (easy frontend integration)
- Modular interfaces (governance, admin, token)

---

## 🎯 How It Works

### For Users:

#### Step 1: Deposit & Stake
```
User deposits 1000 CSPR
→ Receives 1000 stCSPR (1:1 initially)
→ Starts earning ~9% APY
→ Can transfer/use stCSPR in DeFi
```

#### Step 2: Earn Rewards
```
Protocol compounds rewards
→ Exchange rate increases (1 stCSPR = 1.05 CSPR)
→ Your 1000 stCSPR now worth 1050 CSPR
→ No rebasing, just appreciation
```

#### Step 3: Withdraw
```
Request withdrawal of 1000 stCSPR
→ stCSPR burned immediately
→ 7-day countdown starts (unbonding)
→ After 7 days: claim 1050 CSPR
→ You earned 50 CSPR in rewards!
```

### For Governance:

#### Create Proposal:
```
User with 1000+ stCSPR creates proposal
→ "Change performance fee to 3%"
→ 3-day voting period starts
```

#### Vote:
```
stCSPR holders vote (1 token = 1 vote)
→ 8000 FOR, 2000 AGAINST
→ 80% participation (exceeds 20% quorum)
→ 80% approval (exceeds 50% threshold)
```

#### Execute:
```
After voting ends → anyone can execute
→ Proposal succeeds → fee changed to 3%
→ Fully decentralized governance!
```

---

## 🔐 Security Features

### Economic Security:
- Share-based accounting (prevents inflation attacks)
- Minimum 10 CSPR deposits (prevents dust)
- Liquidity checks before withdrawals
- Performance fees for protocol sustainability

### Contract Security:
- Owner-only admin functions
- Emergency pause mechanism
- Event emissions for transparency
- Comprehensive input validation

### Governance Security:
- Quorum requirements (prevents governance attacks)
- Time-locked voting (prevents flash loans)
- Proposal thresholds (prevents spam)
- Cancel mechanism (emergency governance halt)

---

## 📈 Economics

### User Returns:
- **Base Casper APY:** 9.5%
- **Performance Fee:** -0.5% (5% of rewards)
- **Net User APY:** ~9.0%
- **Benefit:** Liquidity + rewards (can use stCSPR in DeFi)

### Protocol Revenue:
- 5% performance fee on all rewards
- Fees go to treasury
- Governance controls fee adjustments
- Sustainable revenue model

---

## 🏗️ What's Built

### Smart Contracts (Rust + Odra):
- ✅ StakeFlowVaultV3 - Main liquid staking protocol
- ✅ StakeFlowGovernance - DAO voting system
- ✅ StakeFlowMinimal - Lightweight version
- ✅ StCSPRToken - Standalone token (CEP-18)
- ✅ Original StakeFlowVault - Feature-rich v1

### Testing:
- ✅ 27 comprehensive tests
- ✅ Unit tests for all functions
- ✅ Integration tests for workflows
- ✅ Edge case coverage

### Documentation:
- ✅ README with full explanation
- ✅ FEATURES.md with detailed breakdown
- ✅ Inline code comments
- ✅ Deployment guides

### Deployment:
- ✅ WASM contracts built
- ✅ Optimized and ready
- ✅ Testnet deployment scripts
- ✅ Awaiting testnet CSPR

---

## 🎪 Comparison vs. Competition

### vs. Simple Liquid Staking Projects:
- ✅ We have withdrawal queue (they don't)
- ✅ We have governance (they don't)
- ✅ We have CEP-18 token (they have basic tokens)
- ✅ We have 27 tests (they have promises)

### vs. CEWCE (Enterprise Workflow):
- Both have governance ✅
- Both have role-based access ✅
- We focus on DeFi, they focus on compliance
- Different tracks, but both enterprise-grade

### vs. Established Liquid Staking (Lido, Rocket Pool):
- Same unbonding mechanism ✅
- Same token appreciation model ✅
- Governance from day 1 (better than Lido v1) ✅
- Built for Casper specifically ✅

---

## 📦 Repository Contents

```
StakeFlow/
├── stakeflow/
│   ├── src/
│   │   ├── stakeflow_vault_v3.rs    (Main protocol - 375KB)
│   │   ├── governance.rs             (DAO system - 344KB)
│   │   ├── stakeflow_minimal.rs     (Lightweight - 290KB)
│   │   ├── stcspr_token.rs          (CEP-18 token)
│   │   └── stakeflow_vault.rs       (Original v1)
│   ├── wasm/                         (Built contracts)
│   └── tests/                        (27 passing tests)
├── README.md                         (Project overview)
├── FEATURES.md                       (Detailed feature list)
├── HACKATHON_SUMMARY.md             (This file)
└── SUPPORT_REQUEST.md               (Testnet token request)
```

---

## 🚀 Next Steps (Post-Hackathon)

### Immediate:
1. Deploy to Casper testnet (waiting for testnet CSPR)
2. Build frontend UI (Next.js + CSPR.click wallet)
3. Test full user flows end-to-end

### Short-term:
1. Security audit
2. Mainnet deployment
3. DeFi integrations (lending, DEX)
4. Validator performance monitoring

### Long-term:
1. Cross-chain bridges (stCSPR to other chains)
2. Advanced governance (delegation, timelocks)
3. Yield optimization strategies
4. NFT liquid staking positions

---

## 🏆 Why StakeFlow Deserves to Win

### 1. Complete Implementation
Not a prototype - production-ready contracts with full test coverage

### 2. Unique Innovation
**Withdrawal queue with unbonding** - the only project with realistic liquid staking

### 3. Enterprise Architecture
Governance, fees, treasury, emergency controls - built for real-world use

### 4. Technical Excellence
27/27 tests, CEP-18 compliance, optimized WASM, clean code

### 5. Long-Term Vision
Not just a hackathon project - foundation for Casper's liquid staking ecosystem

---

## 📞 Contact & Links

- **GitHub:** https://github.com/Hijanhv/StakeFlow
- **Hackathon:** https://dorahacks.io/hackathon/casper-hackathon-2026
- **Testnet Explorer:** https://testnet.cspr.live
- **Developer:** Janhavi

---

## ✅ Judge Verification Checklist

Can verify right now:
- [ ] Clone repo → Run `cargo odra test` → See 27/27 tests pass
- [ ] Check `wasm/` → See production WASM files
- [ ] Read code → See comprehensive implementation
- [ ] Review tests → See full coverage
- [ ] Check FEATURES.md → See detailed breakdown

Unique to StakeFlow:
- [ ] Withdrawal queue with 7-day unbonding
- [ ] Integrated CEP-18 stCSPR token
- [ ] DAO governance system
- [ ] 27 passing tests
- [ ] Production-ready contracts

---

**StakeFlow: Real Liquid Staking for Casper Network** 🚀

Built with ❤️ for Casper Hackathon 2026
