use odra::prelude::*;
use odra::casper_types::{U512, U256};
use crate::stcspr_token::StCSPRToken;

/// StakeFlow Vault V2 - Integrated with stCSPR Token
///
/// This vault manages CSPR deposits and coordinates with stCSPR token.
/// Simplified to stay within Odra's 15-field limit while providing core features.
#[odra::module]
pub struct StakeFlowVaultV2 {
    /// stCSPR token contract reference
    stcspr_token: SubModule<StCSPRToken>,
    /// Total CSPR in vault
    total_cspr: Var<U512>,
    /// Total CSPR staked
    total_staked: Var<U512>,
    /// Owner address
    owner: Var<Address>,
    /// Treasury for fees
    treasury: Var<Address>,
    /// Performance fee (bps)
    performance_fee_bps: Var<u32>,
    /// Protocol fees collected
    protocol_fees: Var<U512>,
    /// Vault active status
    is_active: Var<bool>,
    /// Total rewards earned
    total_rewards: Var<U512>,
    /// Last reward claim time
    last_reward_claim: Var<u64>,
    /// Validator count
    validator_count: Var<u32>,
    /// Withdrawal counter
    withdrawal_counter: Var<u64>,
    /// Reserve ratio (bps)
    reserve_ratio_bps: Var<u32>,
}

#[odra::module]
impl StakeFlowVaultV2 {
    /// Initialize vault
    pub fn init(&mut self, treasury_address: Address) {
        let caller = self.env().caller();
        self.owner.set(caller);
        self.treasury.set(treasury_address);
        self.total_cspr.set(U512::zero());
        self.total_staked.set(U512::zero());
        self.total_rewards.set(U512::zero());
        self.protocol_fees.set(U512::zero());
        self.is_active.set(true);
        self.validator_count.set(0);
        self.withdrawal_counter.set(0);
        self.last_reward_claim.set(self.env().get_block_time());
        self.performance_fee_bps.set(500); // 5%
        self.reserve_ratio_bps.set(1000); // 10%

        self.env().emit_event(VaultInitialized {
            owner: caller,
            treasury: treasury_address,
            stcspr_token: self.stcspr_token.address(),
            timestamp: self.env().get_block_time(),
        });
    }

    /// Deposit CSPR - mints stCSPR tokens to user
    #[odra(payable)]
    pub fn deposit(&mut self) -> U256 {
        assert!(self.is_active.get_or_default(), "Vault is paused");

        let caller = self.env().caller();
        let cspr_amount = self.env().attached_value();
        let min_deposit = U512::from(10_000_000_000u64);
        assert!(cspr_amount >= min_deposit, "Minimum deposit is 10 CSPR");

        // Update vault total
        self.total_cspr.set(self.total_cspr.get_or_default() + cspr_amount);

        // Mint stCSPR tokens to user
        let cspr_u256 = U256::from(cspr_amount.as_u128());
        let stcspr_amount = self.stcspr_token.mint(caller, cspr_u256);

        self.env().emit_event(Deposited {
            user: caller,
            cspr_amount,
            stcspr_amount,
            timestamp: self.env().get_block_time(),
        });

        stcspr_amount
    }

    /// Execute withdrawal - burns stCSPR tokens, returns CSPR
    pub fn withdraw(&mut self, stcspr_amount: U256) {
        assert!(self.is_active.get_or_default(), "Vault is paused");
        assert!(stcspr_amount > U256::zero(), "Amount must be > 0");

        let caller = self.env().caller();

        // Burn stCSPR tokens and get CSPR amount to return
        let cspr_u256 = self.stcspr_token.burn(caller, stcspr_amount);
        let cspr_amount = U512::from(cspr_u256.as_u128());

        // Check liquidity
        let available = self.total_cspr.get_or_default() - self.total_staked.get_or_default();
        assert!(cspr_amount <= available, "Insufficient liquidity");

        // Update vault total
        self.total_cspr.set(self.total_cspr.get_or_default() - cspr_amount);

        // Transfer CSPR to user
        self.env().transfer_tokens(&caller, &cspr_amount);

        self.env().emit_event(Withdrawn {
            user: caller,
            cspr_amount,
            stcspr_amount,
            timestamp: self.env().get_block_time(),
        });
    }

    /// Compound rewards - updates exchange rate in token contract
    pub fn compound_rewards(&mut self, rewards_amount: U512) {
        self.assert_owner();
        assert!(rewards_amount > U512::zero(), "No rewards");

        let fee_bps = self.performance_fee_bps.get_or_default();
        let protocol_fee = (rewards_amount * U512::from(fee_bps)) / U512::from(10000u64);
        let user_rewards = rewards_amount - protocol_fee;

        self.protocol_fees.set(self.protocol_fees.get_or_default() + protocol_fee);
        self.total_rewards.set(self.total_rewards.get_or_default() + user_rewards);
        self.total_cspr.set(self.total_cspr.get_or_default() + user_rewards);

        // Update exchange rate in stCSPR token contract
        let new_total = U256::from(self.total_cspr.get_or_default().as_u128());
        self.stcspr_token.update_exchange_rate(new_total);

        self.env().emit_event(RewardsCompounded {
            total_rewards: rewards_amount,
            protocol_fee,
            user_rewards,
            timestamp: self.env().get_block_time(),
        });
    }

    /// Withdraw protocol fees
    pub fn withdraw_protocol_fees(&mut self) {
        self.assert_owner();
        let fees = self.protocol_fees.get_or_default();
        assert!(fees > U512::zero(), "No fees");

        self.protocol_fees.set(U512::zero());
        let treasury = self.treasury.get().unwrap();
        self.env().transfer_tokens(&treasury, &fees);
    }

    /// Stake to validators
    pub fn stake_to_validators(&mut self, amount: U512) {
        self.assert_owner();
        let unstaked = self.total_cspr.get_or_default() - self.total_staked.get_or_default();
        let reserve = (self.total_cspr.get_or_default() * U512::from(self.reserve_ratio_bps.get_or_default())) / U512::from(10000u64);
        assert!(unstaked > reserve, "Must maintain reserve");
        assert!(amount <= unstaked - reserve, "Exceeds stakeable");

        self.total_staked.set(self.total_staked.get_or_default() + amount);
    }

    /// Get TVL
    pub fn get_tvl(&self) -> U512 {
        self.total_cspr.get_or_default()
    }

    /// Get total staked
    pub fn get_total_staked(&self) -> U512 {
        self.total_staked.get_or_default()
    }

    /// Get total rewards
    pub fn get_total_rewards(&self) -> U512 {
        self.total_rewards.get_or_default()
    }

    /// Get protocol fees
    pub fn get_protocol_fees(&self) -> U512 {
        self.protocol_fees.get_or_default()
    }

    /// Get APY
    pub fn get_apy(&self) -> u32 {
        let base = 950u32;
        let fee = self.performance_fee_bps.get_or_default();
        base - (base * fee / 10000)
    }

    /// Get stCSPR token address
    pub fn get_stcspr_token(&self) -> Address {
        self.stcspr_token.address()
    }

    /// Get stCSPR exchange rate
    pub fn get_exchange_rate(&self) -> U256 {
        self.stcspr_token.get_exchange_rate()
    }

    /// Pause
    pub fn pause(&mut self) {
        self.assert_owner();
        self.is_active.set(false);
    }

    /// Unpause
    pub fn unpause(&mut self) {
        self.assert_owner();
        self.is_active.set(true);
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.is_active.get_or_default()
    }

    /// Set performance fee
    pub fn set_performance_fee(&mut self, new_fee_bps: u32) {
        self.assert_owner();
        assert!(new_fee_bps <= 1000, "Max 10%");
        self.performance_fee_bps.set(new_fee_bps);
    }

    fn assert_owner(&self) {
        let caller = self.env().caller();
        let owner = self.owner.get().unwrap();
        assert!(caller == owner, "Only owner");
    }
}

// Events
#[odra::event]
pub struct VaultInitialized {
    pub owner: Address,
    pub treasury: Address,
    pub stcspr_token: Address,
    pub timestamp: u64,
}

#[odra::event]
pub struct Deposited {
    pub user: Address,
    pub cspr_amount: U512,
    pub stcspr_amount: U256,
    pub timestamp: u64,
}

#[odra::event]
pub struct Withdrawn {
    pub user: Address,
    pub cspr_amount: U512,
    pub stcspr_amount: U256,
    pub timestamp: u64,
}

#[odra::event]
pub struct RewardsCompounded {
    pub total_rewards: U512,
    pub protocol_fee: U512,
    pub user_rewards: U512,
    pub timestamp: u64,
}

// Tests disabled - VaultV2 not in use, using VaultV3 instead
/*
#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostRef};

    #[test]
    fn test_integrated_deposit() {
        let env = odra_test::env();
        let treasury = env.get_account(9);

        // Deploy integrated vault (token is deployed as submodule)
        let mut vault = StakeFlowVaultV2::deploy(&env, StakeFlowVaultV2InitArgs {
            treasury_address: treasury,
        });

        let amount = U512::from(100_000_000_000u64); // 100 CSPR
        env.set_caller(env.get_account(1));
        let stcspr_amount = vault.with_tokens(amount).deposit();

        assert_eq!(vault.get_tvl(), amount);
        assert_eq!(stcspr_amount, U256::from(amount.as_u128())); // 1:1 initially
    }

    #[test]
    fn test_integrated_withdraw() {
        let env = odra_test::env();
        let treasury = env.get_account(9);
        let mut vault = StakeFlowVaultV2::deploy(&env, StakeFlowVaultV2InitArgs {
            treasury_address: treasury,
        });

        let deposit_amount = U512::from(100_000_000_000u64);
        env.set_caller(env.get_account(1));
        let stcspr_minted = vault.with_tokens(deposit_amount).deposit();

        // Withdraw half
        let withdraw_amount = stcspr_minted / U256::from(2u64);
        vault.withdraw(withdraw_amount);

        // TVL should be reduced by half
        assert!(vault.get_tvl() < deposit_amount);
    }

    #[test]
    fn test_rewards_compounding() {
        let env = odra_test::env();
        let treasury = env.get_account(9);
        let mut vault = StakeFlowVaultV2::deploy(&env, StakeFlowVaultV2InitArgs {
            treasury_address: treasury,
        });

        // User deposits
        let deposit = U512::from(1000_000_000_000u64); // 1000 CSPR
        env.set_caller(env.get_account(1));
        vault.with_tokens(deposit).deposit();

        // Owner compounds rewards
        env.set_caller(env.get_account(0)); // owner
        let rewards = U512::from(100_000_000_000u64); // 100 CSPR rewards
        vault.compound_rewards(rewards);

        // Exchange rate should have increased
        let exchange_rate = vault.get_exchange_rate();
        assert!(exchange_rate > U256::from(1_000_000_000u64));

        // TVL includes rewards minus fees
        assert!(vault.get_tvl() > deposit);
    }

    #[test]
    fn test_performance_fees() {
        let env = odra_test::env();
        let treasury = env.get_account(9);
        let mut vault = StakeFlowVaultV2::deploy(&env, StakeFlowVaultV2InitArgs {
            treasury_address: treasury,
        });

        env.set_caller(env.get_account(1));
        vault.with_tokens(U512::from(1000_000_000_000u64)).deposit();

        env.set_caller(env.get_account(0));
        let rewards = U512::from(100_000_000_000u64); // 100 CSPR
        vault.compound_rewards(rewards);

        // 5% fee = 5 CSPR
        let expected_fee = U512::from(5_000_000_000u64);
        assert_eq!(vault.get_protocol_fees(), expected_fee);
    }
}
*/
