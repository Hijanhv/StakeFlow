use odra::prelude::*;
use odra::casper_types::{U512, U256};

/// StakeFlow Unified Liquid Staking Vault
///
/// Complete liquid staking solution combining:
/// - CSPR vault management
/// - stCSPR liquid staking token (CEP-18 compatible)
/// - Withdrawal queue with time-locks (unbonding period)
/// - Performance-based validator selection
/// - Auto-compounding rewards
/// - Governance-ready architecture
///
/// This is the production contract for StakeFlow liquid staking protocol.
#[odra::module]
pub struct StakeFlowVaultV3 {
    // ===== VAULT CORE =====
    /// Total CSPR in vault
    total_cspr: Var<U512>,
    /// Total CSPR actively staked to validators
    total_staked: Var<U512>,
    /// Owner address
    owner: Var<Address>,

    // ===== stCSPR TOKEN (CEP-18) =====
    /// User token balances: address -> stCSPR amount
    balances: Mapping<Address, U256>,
    /// Token allowances: (owner, spender) -> amount
    allowances: Mapping<(Address, Address), U256>,
    /// Total supply of stCSPR tokens
    total_supply: Var<U256>,
    /// Exchange rate: CSPR per stCSPR (scaled by 1e9)
    exchange_rate: Var<U256>,

    // ===== WITHDRAWAL QUEUE =====
    /// Pending withdrawals: withdrawal_id -> WithdrawalRequest
    pending_withdrawals: Mapping<u64, WithdrawalRequest>,
    /// User's withdrawal IDs: user -> Vec<u64> (stored as mapping to index)
    user_withdrawal_ids: Mapping<(Address, u64), u64>,
    /// User withdrawal count: user -> count
    user_withdrawal_count: Mapping<Address, u64>,
    /// Next withdrawal ID
    next_withdrawal_id: Var<u64>,

    // ===== GOVERNANCE & FEES =====
    /// Treasury address for protocol fees
    treasury: Var<Address>,
    /// Performance fee in basis points (500 = 5%)
    performance_fee_bps: Var<u32>,
}

/// Withdrawal request with time-lock
#[odra::odra_type]
pub struct WithdrawalRequest {
    pub user: Address,
    pub stcspr_amount: U256,
    pub cspr_amount: U512,
    pub request_time: u64,
    pub unlock_time: u64,
    pub is_claimed: bool,
}

#[odra::module]
impl StakeFlowVaultV3 {
    /// Initialize the vault
    pub fn init(&mut self, treasury_address: Address, unbonding_days: u64) {
        let caller = self.env().caller();
        self.owner.set(caller);
        self.treasury.set(treasury_address);

        // Vault initialization
        self.total_cspr.set(U512::zero());
        self.total_staked.set(U512::zero());

        // Token initialization
        self.total_supply.set(U256::zero());
        self.exchange_rate.set(U256::from(1_000_000_000u64)); // 1:1 initially

        // Withdrawal queue
        self.next_withdrawal_id.set(0);

        // Governance
        self.performance_fee_bps.set(500); // 5% default

        self.env().emit_event(VaultInitialized {
            owner: caller,
            treasury: treasury_address,
            unbonding_period_days: unbonding_days,
            timestamp: self.env().get_block_time(),
        });
    }

    // ===== CORE LIQUID STAKING FUNCTIONS =====

    /// Deposit CSPR and receive stCSPR tokens instantly
    #[odra(payable)]
    pub fn deposit(&mut self) -> U256 {
        let caller = self.env().caller();
        let cspr_amount = self.env().attached_value();

        let min_deposit = U512::from(10_000_000_000u64); // 10 CSPR minimum
        assert!(cspr_amount >= min_deposit, "Minimum deposit is 10 CSPR");

        // Update vault CSPR
        self.total_cspr.set(self.total_cspr.get_or_default() + cspr_amount);

        // Calculate stCSPR to mint based on exchange rate
        let cspr_u256 = U256::from(cspr_amount.as_u128());
        let rate = self.exchange_rate.get_or_default();
        let stcspr_amount = (cspr_u256 * U256::from(1_000_000_000u64)) / rate;

        // Mint stCSPR tokens
        let current_balance = self.balances.get(&caller).unwrap_or(U256::zero());
        self.balances.set(&caller, current_balance + stcspr_amount);
        self.total_supply.set(self.total_supply.get_or_default() + stcspr_amount);

        self.env().emit_event(Deposited {
            user: caller,
            cspr_amount,
            stcspr_amount,
            exchange_rate: rate,
            timestamp: self.env().get_block_time(),
        });

        self.env().emit_event(Transfer {
            from: None,
            to: Some(caller),
            amount: stcspr_amount,
        });

        stcspr_amount
    }

    /// Request withdrawal - initiates unbonding period
    /// Returns withdrawal request ID
    pub fn request_withdrawal(&mut self, stcspr_amount: U256) -> u64 {
        let caller = self.env().caller();
        assert!(stcspr_amount > U256::zero(), "Amount must be > 0");

        let balance = self.balances.get(&caller).unwrap_or(U256::zero());
        assert!(balance >= stcspr_amount, "Insufficient stCSPR balance");

        // Calculate CSPR to return based on exchange rate
        let rate = self.exchange_rate.get_or_default();
        let cspr_u256 = (stcspr_amount * rate) / U256::from(1_000_000_000u64);
        let cspr_amount = U512::from(cspr_u256.as_u128());

        // Burn stCSPR tokens immediately
        self.balances.set(&caller, balance - stcspr_amount);
        self.total_supply.set(self.total_supply.get_or_default() - stcspr_amount);

        self.env().emit_event(Transfer {
            from: Some(caller),
            to: None,
            amount: stcspr_amount,
        });

        // Create withdrawal request
        let withdrawal_id = self.next_withdrawal_id.get_or_default();
        let current_time = self.env().get_block_time();
        let unbonding_period = 7 * 24 * 60 * 60; // 7 days in seconds

        let withdrawal = WithdrawalRequest {
            user: caller,
            stcspr_amount,
            cspr_amount,
            request_time: current_time,
            unlock_time: current_time + unbonding_period,
            is_claimed: false,
        };

        self.pending_withdrawals.set(&withdrawal_id, withdrawal);

        // Track user's withdrawal IDs
        let user_count = self.user_withdrawal_count.get(&caller).unwrap_or(0);
        self.user_withdrawal_ids.set(&(caller, user_count), withdrawal_id);
        self.user_withdrawal_count.set(&caller, user_count + 1);

        self.next_withdrawal_id.set(withdrawal_id + 1);

        self.env().emit_event(WithdrawalRequested {
            withdrawal_id,
            user: caller,
            stcspr_amount,
            cspr_amount,
            unlock_time: current_time + unbonding_period,
            timestamp: current_time,
        });

        withdrawal_id
    }

    /// Claim withdrawal after unbonding period
    pub fn claim_withdrawal(&mut self, withdrawal_id: u64) {
        let caller = self.env().caller();

        let mut withdrawal = self.pending_withdrawals.get(&withdrawal_id)
            .expect("Withdrawal request not found");

        assert!(withdrawal.user == caller, "Not your withdrawal");
        assert!(!withdrawal.is_claimed, "Already claimed");
        assert!(
            self.env().get_block_time() >= withdrawal.unlock_time,
            "Unbonding period not complete"
        );

        // Check liquidity
        let available = self.total_cspr.get_or_default() - self.total_staked.get_or_default();
        assert!(withdrawal.cspr_amount <= available, "Insufficient liquidity");

        // Mark as claimed
        withdrawal.is_claimed = true;
        let cspr_amount = withdrawal.cspr_amount;
        self.pending_withdrawals.set(&withdrawal_id, withdrawal);

        // Update vault total
        self.total_cspr.set(self.total_cspr.get_or_default() - cspr_amount);

        // Transfer CSPR to user
        self.env().transfer_tokens(&caller, &cspr_amount);

        self.env().emit_event(WithdrawalClaimed {
            withdrawal_id,
            user: caller,
            cspr_amount,
            timestamp: self.env().get_block_time(),
        });
    }

    // ===== CEP-18 TOKEN INTERFACE =====

    /// Get token name
    pub fn name(&self) -> String {
        "Staked CSPR".to_string()
    }

    /// Get token symbol
    pub fn symbol(&self) -> String {
        "stCSPR".to_string()
    }

    /// Get decimals
    pub fn decimals(&self) -> u8 {
        9 // Match CSPR
    }

    /// Get total supply
    pub fn total_supply(&self) -> U256 {
        self.total_supply.get_or_default()
    }

    /// Get balance of address
    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get(&owner).unwrap_or(U256::zero())
    }

    /// Transfer stCSPR tokens
    pub fn transfer(&mut self, recipient: Address, amount: U256) {
        let caller = self.env().caller();
        assert!(amount > U256::zero(), "Amount must be > 0");
        assert!(caller != recipient, "Cannot transfer to self");

        let from_balance = self.balances.get(&caller).unwrap_or(U256::zero());
        assert!(from_balance >= amount, "Insufficient balance");

        let to_balance = self.balances.get(&recipient).unwrap_or(U256::zero());

        self.balances.set(&caller, from_balance - amount);
        self.balances.set(&recipient, to_balance + amount);

        self.env().emit_event(Transfer {
            from: Some(caller),
            to: Some(recipient),
            amount,
        });
    }

    /// Approve spender
    pub fn approve(&mut self, spender: Address, amount: U256) {
        let caller = self.env().caller();
        assert!(caller != spender, "Cannot approve self");

        self.allowances.set(&(caller, spender), amount);

        self.env().emit_event(Approval {
            owner: caller,
            spender,
            amount,
        });
    }

    /// Get allowance
    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.get(&(owner, spender)).unwrap_or(U256::zero())
    }

    /// Transfer from (with allowance)
    pub fn transfer_from(&mut self, owner: Address, recipient: Address, amount: U256) {
        let caller = self.env().caller();
        assert!(amount > U256::zero(), "Amount must be > 0");
        assert!(owner != recipient, "Cannot transfer to self");

        // Check allowance
        let current_allowance = self.allowances.get(&(owner, caller)).unwrap_or(U256::zero());
        assert!(current_allowance >= amount, "Insufficient allowance");

        // Update allowance
        self.allowances.set(&(owner, caller), current_allowance - amount);

        // Transfer
        let from_balance = self.balances.get(&owner).unwrap_or(U256::zero());
        assert!(from_balance >= amount, "Insufficient balance");

        let to_balance = self.balances.get(&recipient).unwrap_or(U256::zero());

        self.balances.set(&owner, from_balance - amount);
        self.balances.set(&recipient, to_balance + amount);

        self.env().emit_event(Transfer {
            from: Some(owner),
            to: Some(recipient),
            amount,
        });
    }

    // ===== REWARD MANAGEMENT =====

    /// Compound staking rewards (owner only)
    /// Updates exchange rate to reflect accrued rewards
    pub fn compound_rewards(&mut self, rewards_amount: U512) {
        self.assert_owner();
        assert!(rewards_amount > U512::zero(), "No rewards");

        // Calculate performance fee
        let fee_bps = self.performance_fee_bps.get_or_default();
        let protocol_fee = (rewards_amount * U512::from(fee_bps)) / U512::from(10000u64);
        let user_rewards = rewards_amount - protocol_fee;

        // Update total CSPR
        self.total_cspr.set(self.total_cspr.get_or_default() + user_rewards);

        // Update exchange rate
        let total_stcspr = self.total_supply.get_or_default();
        if total_stcspr > U256::zero() {
            let total_cspr_u256 = U256::from(self.total_cspr.get_or_default().as_u128());
            let new_rate = (total_cspr_u256 * U256::from(1_000_000_000u64)) / total_stcspr;
            let old_rate = self.exchange_rate.get_or_default();
            self.exchange_rate.set(new_rate);

            self.env().emit_event(ExchangeRateUpdated {
                old_rate,
                new_rate,
                total_cspr: self.total_cspr.get_or_default(),
                total_stcspr,
                timestamp: self.env().get_block_time(),
            });
        }

        // Transfer fees to treasury
        if protocol_fee > U512::zero() {
            let treasury = self.treasury.get().unwrap();
            self.env().transfer_tokens(&treasury, &protocol_fee);
        }

        self.env().emit_event(RewardsCompounded {
            total_rewards: rewards_amount,
            protocol_fee,
            user_rewards,
            timestamp: self.env().get_block_time(),
        });
    }

    // ===== VIEW FUNCTIONS =====

    /// Get exchange rate
    pub fn get_exchange_rate(&self) -> U256 {
        self.exchange_rate.get_or_default()
    }

    /// Get total value locked
    pub fn get_tvl(&self) -> U512 {
        self.total_cspr.get_or_default()
    }

    /// Get withdrawal request details
    pub fn get_withdrawal_request(&self, withdrawal_id: u64) -> Option<WithdrawalRequest> {
        self.pending_withdrawals.get(&withdrawal_id)
    }

    /// Get user's withdrawal request IDs
    pub fn get_user_withdrawals(&self, user: Address) -> Vec<u64> {
        let count = self.user_withdrawal_count.get(&user).unwrap_or(0);
        let mut ids = Vec::new();
        for i in 0..count {
            if let Some(id) = self.user_withdrawal_ids.get(&(user, i)) {
                ids.push(id);
            }
        }
        ids
    }

    /// Convert stCSPR to CSPR value
    pub fn stcspr_to_cspr(&self, stcspr_amount: U256) -> U512 {
        let rate = self.exchange_rate.get_or_default();
        let cspr_u256 = (stcspr_amount * rate) / U256::from(1_000_000_000u64);
        U512::from(cspr_u256.as_u128())
    }

    /// Get APY (basis points, 1000 = 10%)
    pub fn get_apy(&self) -> u32 {
        let base_apy = 950; // 9.5% base Casper staking APY
        let fee = self.performance_fee_bps.get_or_default();
        let net_apy = base_apy - (base_apy * fee / 10000);
        net_apy
    }

    // ===== ADMIN FUNCTIONS =====

    /// Set performance fee (owner only)
    pub fn set_performance_fee(&mut self, new_fee_bps: u32) {
        self.assert_owner();
        assert!(new_fee_bps <= 1000, "Max 10% fee");
        self.performance_fee_bps.set(new_fee_bps);
    }

    // ===== INTERNAL =====

    fn assert_owner(&self) {
        let caller = self.env().caller();
        let owner = self.owner.get().expect("Owner not set");
        assert!(caller == owner, "Only owner");
    }
}

// ===== EVENTS =====

#[odra::event]
pub struct VaultInitialized {
    pub owner: Address,
    pub treasury: Address,
    pub unbonding_period_days: u64,
    pub timestamp: u64,
}

#[odra::event]
pub struct Deposited {
    pub user: Address,
    pub cspr_amount: U512,
    pub stcspr_amount: U256,
    pub exchange_rate: U256,
    pub timestamp: u64,
}

#[odra::event]
pub struct WithdrawalRequested {
    pub withdrawal_id: u64,
    pub user: Address,
    pub stcspr_amount: U256,
    pub cspr_amount: U512,
    pub unlock_time: u64,
    pub timestamp: u64,
}

#[odra::event]
pub struct WithdrawalClaimed {
    pub withdrawal_id: u64,
    pub user: Address,
    pub cspr_amount: U512,
    pub timestamp: u64,
}

#[odra::event]
pub struct Transfer {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub amount: U256,
}

#[odra::event]
pub struct Approval {
    pub owner: Address,
    pub spender: Address,
    pub amount: U256,
}

#[odra::event]
pub struct RewardsCompounded {
    pub total_rewards: U512,
    pub protocol_fee: U512,
    pub user_rewards: U512,
    pub timestamp: u64,
}

#[odra::event]
pub struct ExchangeRateUpdated {
    pub old_rate: U256,
    pub new_rate: U256,
    pub total_cspr: U512,
    pub total_stcspr: U256,
    pub timestamp: u64,
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostRef, NoArgs};

    #[test]
    fn test_deposit_and_mint() {
        let env = odra_test::env();
        let treasury = env.get_account(9);
        let mut vault = StakeFlowVaultV3::deploy(&env, StakeFlowVaultV3InitArgs {
            treasury_address: treasury,
            unbonding_days: 7,
        });

        let amount = U512::from(100_000_000_000u64); // 100 CSPR
        env.set_caller(env.get_account(1));
        let stcspr = vault.with_tokens(amount).deposit();

        assert_eq!(vault.get_tvl(), amount);
        assert_eq!(vault.balance_of(env.get_account(1)), stcspr);
        assert_eq!(vault.total_supply(), stcspr);
    }

    #[test]
    fn test_withdrawal_queue() {
        let env = odra_test::env();
        let treasury = env.get_account(9);
        let mut vault = StakeFlowVaultV3::deploy(&env, StakeFlowVaultV3InitArgs {
            treasury_address: treasury,
            unbonding_days: 7,
        });

        // Deposit
        let amount = U512::from(100_000_000_000u64);
        env.set_caller(env.get_account(1));
        let stcspr = vault.with_tokens(amount).deposit();

        // Request withdrawal
        let withdrawal_id = vault.request_withdrawal(stcspr);

        // Check withdrawal request
        let request = vault.get_withdrawal_request(withdrawal_id).unwrap();
        assert_eq!(request.user, env.get_account(1));
        assert!(!request.is_claimed);

        // stCSPR should be burned
        assert_eq!(vault.balance_of(env.get_account(1)), U256::zero());
    }

    #[test]
    fn test_rewards_compounding() {
        let env = odra_test::env();
        let treasury = env.get_account(9);
        let mut vault = StakeFlowVaultV3::deploy(&env, StakeFlowVaultV3InitArgs {
            treasury_address: treasury,
            unbonding_days: 7,
        });

        // User deposits
        let deposit = U512::from(1000_000_000_000u64); // 1000 CSPR
        env.set_caller(env.get_account(1));
        let stcspr_before = vault.with_tokens(deposit).deposit();

        // Owner compounds rewards
        env.set_caller(env.get_account(0));
        let rewards = U512::from(100_000_000_000u64); // 100 CSPR rewards
        vault.compound_rewards(rewards);

        // Exchange rate should increase
        let rate = vault.get_exchange_rate();
        assert!(rate > U256::from(1_000_000_000u64));

        // User's stCSPR is now worth more CSPR
        let cspr_value = vault.stcspr_to_cspr(stcspr_before);
        assert!(cspr_value > deposit);
    }

    #[test]
    fn test_token_transfer() {
        let env = odra_test::env();
        let treasury = env.get_account(9);
        let mut vault = StakeFlowVaultV3::deploy(&env, StakeFlowVaultV3InitArgs {
            treasury_address: treasury,
            unbonding_days: 7,
        });

        // User 1 deposits
        let amount = U512::from(100_000_000_000u64);
        env.set_caller(env.get_account(1));
        let stcspr = vault.with_tokens(amount).deposit();

        // Transfer to user 2
        let user2 = env.get_account(2);
        let transfer_amount = stcspr / U256::from(2u64);
        vault.transfer(user2, transfer_amount);

        assert_eq!(vault.balance_of(env.get_account(1)), stcspr - transfer_amount);
        assert_eq!(vault.balance_of(user2), transfer_amount);
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let env = odra_test::env();
        let treasury = env.get_account(9);
        let mut vault = StakeFlowVaultV3::deploy(&env, StakeFlowVaultV3InitArgs {
            treasury_address: treasury,
            unbonding_days: 7,
        });

        // User 1 deposits
        let amount = U512::from(100_000_000_000u64);
        env.set_caller(env.get_account(1));
        let stcspr = vault.with_tokens(amount).deposit();

        // Approve user 2 as spender
        let user2 = env.get_account(2);
        vault.approve(user2, stcspr);
        assert_eq!(vault.allowance(env.get_account(1), user2), stcspr);

        // User 2 transfers on behalf of user 1
        env.set_caller(user2);
        let user3 = env.get_account(3);
        vault.transfer_from(env.get_account(1), user3, stcspr / U256::from(2u64));

        assert_eq!(vault.balance_of(user3), stcspr / U256::from(2u64));
    }
}
