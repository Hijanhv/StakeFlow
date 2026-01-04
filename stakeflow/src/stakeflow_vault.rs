use odra::prelude::*;
use odra::casper_types::U512;

/// StakeFlow Vault - Liquid Staking & DeFi Yield Optimization Platform
///
/// This contract accepts CSPR deposits, stakes them through Casper's native liquid staking,
/// receives sCSPR tokens, and deploys them into DeFi yield strategies.
#[odra::module]
pub struct StakeFlowVault {
    /// Total CSPR deposited by all users
    total_deposits: Var<U512>,
    /// Total sCSPR tokens held by the vault
    total_scspr: Var<U512>,
    /// User deposits: mapping from address to CSPR amount
    user_deposits: Mapping<Address, U512>,
    /// User shares: mapping from address to share amount
    user_shares: Mapping<Address, U512>,
    /// Total shares issued
    total_shares: Var<U512>,
    /// Contract owner
    owner: Var<Address>,
    /// Vault active status
    is_active: Var<bool>,
}

#[odra::module]
impl StakeFlowVault {
    /// Initialize the StakeFlow Vault
    pub fn init(&mut self) {
        let caller = self.env().caller();
        self.owner.set(caller);
        self.total_deposits.set(U512::zero());
        self.total_scspr.set(U512::zero());
        self.total_shares.set(U512::zero());
        self.is_active.set(true);

        self.env().emit_event(VaultInitialized {
            owner: caller,
            timestamp: self.env().get_block_time(),
        });
    }

    /// Deposit CSPR into the vault
    /// Returns the number of shares minted to the depositor
    #[odra(payable)]
    pub fn deposit(&mut self) -> U512 {
        self.assert_vault_active();

        let caller = self.env().caller();
        let amount = self.env().attached_value();

        // Require minimum deposit of 10 CSPR
        let min_deposit = U512::from(10_000_000_000u64); // 10 CSPR in motes
        assert!(amount >= min_deposit, "Minimum deposit is 10 CSPR");

        // Calculate shares to mint
        let shares = self.calculate_shares_for_deposit(amount);

        // Update user deposits
        let current_deposit = self.user_deposits.get(&caller).unwrap_or(U512::zero());
        self.user_deposits.set(&caller, current_deposit + amount);

        // Update user shares
        let current_shares = self.user_shares.get(&caller).unwrap_or(U512::zero());
        self.user_shares.set(&caller, current_shares + shares);

        // Update totals
        self.total_deposits.set(self.total_deposits.get_or_default() + amount);
        self.total_shares.set(self.total_shares.get_or_default() + shares);

        self.env().emit_event(Deposited {
            user: caller,
            amount,
            shares,
            timestamp: self.env().get_block_time(),
        });

        shares
    }

    /// Withdraw CSPR from the vault by burning shares
    pub fn withdraw(&mut self, shares_to_burn: U512) {
        self.assert_vault_active();

        let caller = self.env().caller();
        let user_shares = self.user_shares.get(&caller).unwrap_or(U512::zero());

        assert!(shares_to_burn > U512::zero(), "Must burn at least 1 share");
        assert!(user_shares >= shares_to_burn, "Insufficient shares");

        // Calculate CSPR amount to return
        let cspr_amount = self.calculate_cspr_for_shares(shares_to_burn);

        // Update user shares
        self.user_shares.set(&caller, user_shares - shares_to_burn);

        // Update totals
        self.total_shares.set(self.total_shares.get_or_default() - shares_to_burn);
        self.total_deposits.set(self.total_deposits.get_or_default() - cspr_amount);

        // Transfer CSPR to user
        self.env().transfer_tokens(&caller, &cspr_amount);

        self.env().emit_event(Withdrawn {
            user: caller,
            amount: cspr_amount,
            shares_burned: shares_to_burn,
            timestamp: self.env().get_block_time(),
        });
    }

    /// Get user's share balance
    pub fn get_user_shares(&self, user: Address) -> U512 {
        self.user_shares.get(&user).unwrap_or(U512::zero())
    }

    /// Get user's CSPR deposit
    pub fn get_user_deposit(&self, user: Address) -> U512 {
        self.user_deposits.get(&user).unwrap_or(U512::zero())
    }

    /// Get user's CSPR value (including yields)
    pub fn get_user_value(&self, user: Address) -> U512 {
        let shares = self.user_shares.get(&user).unwrap_or(U512::zero());
        self.calculate_cspr_for_shares(shares)
    }

    /// Get total value locked (TVL)
    pub fn get_tvl(&self) -> U512 {
        self.total_deposits.get_or_default()
    }

    /// Get total shares
    pub fn get_total_shares(&self) -> U512 {
        self.total_shares.get_or_default()
    }

    /// Get current APY (placeholder for now)
    pub fn get_apy(&self) -> U512 {
        // TODO: Calculate actual APY based on yields
        U512::from(8_00u64) // 8% placeholder
    }

    /// Check if vault is active
    pub fn is_active(&self) -> bool {
        self.is_active.get_or_default()
    }

    /// Emergency pause (owner only)
    pub fn pause(&mut self) {
        self.assert_owner();
        self.is_active.set(false);

        self.env().emit_event(VaultPaused {
            timestamp: self.env().get_block_time(),
        });
    }

    /// Unpause (owner only)
    pub fn unpause(&mut self) {
        self.assert_owner();
        self.is_active.set(true);

        self.env().emit_event(VaultUnpaused {
            timestamp: self.env().get_block_time(),
        });
    }

    // ===== INTERNAL FUNCTIONS =====

    /// Calculate shares for a deposit amount
    /// If first deposit, 1:1 ratio, otherwise proportional to existing shares
    fn calculate_shares_for_deposit(&self, amount: U512) -> U512 {
        let total_shares = self.total_shares.get_or_default();
        let total_deposits = self.total_deposits.get_or_default();

        if total_shares == U512::zero() || total_deposits == U512::zero() {
            // First deposit: 1:1 ratio
            amount
        } else {
            // Proportional to existing shares
            (amount * total_shares) / total_deposits
        }
    }

    /// Calculate CSPR amount for shares
    fn calculate_cspr_for_shares(&self, shares: U512) -> U512 {
        let total_shares = self.total_shares.get_or_default();
        let total_deposits = self.total_deposits.get_or_default();

        if total_shares == U512::zero() {
            U512::zero()
        } else {
            (shares * total_deposits) / total_shares
        }
    }

    /// Assert caller is owner
    fn assert_owner(&self) {
        let caller = self.env().caller();
        if let Some(owner) = self.owner.get() {
            assert!(caller == owner, "Only owner can call this function");
        } else {
            panic!("Owner not set");
        }
    }

    /// Assert vault is active
    fn assert_vault_active(&self) {
        assert!(self.is_active.get_or_default(), "Vault is paused");
    }
}

// ===== EVENTS =====

#[odra::event]
pub struct VaultInitialized {
    pub owner: Address,
    pub timestamp: u64,
}

#[odra::event]
pub struct Deposited {
    pub user: Address,
    pub amount: U512,
    pub shares: U512,
    pub timestamp: u64,
}

#[odra::event]
pub struct Withdrawn {
    pub user: Address,
    pub amount: U512,
    pub shares_burned: U512,
    pub timestamp: u64,
}

#[odra::event]
pub struct VaultPaused {
    pub timestamp: u64,
}

#[odra::event]
pub struct VaultUnpaused {
    pub timestamp: u64,
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostRef, NoArgs};

    #[test]
    fn test_initialization() {
        let env = odra_test::env();
        let contract = StakeFlowVault::deploy(&env, NoArgs);

        assert_eq!(contract.get_tvl(), U512::zero());
        assert_eq!(contract.get_total_shares(), U512::zero());
        assert!(contract.is_active());
    }

    #[test]
    fn test_deposit() {
        let env = odra_test::env();
        let mut contract = StakeFlowVault::deploy(&env, NoArgs);

        let deposit_amount = U512::from(100_000_000_000u64); // 100 CSPR

        env.set_caller(env.get_account(1));
        let shares = contract.with_tokens(deposit_amount).deposit();

        assert_eq!(contract.get_total_shares(), shares);
        assert_eq!(contract.get_user_shares(env.get_account(1)), shares);
        assert_eq!(contract.get_tvl(), deposit_amount);
    }

    #[test]
    fn test_withdraw() {
        let env = odra_test::env();
        let mut contract = StakeFlowVault::deploy(&env, NoArgs);

        let deposit_amount = U512::from(100_000_000_000u64); // 100 CSPR

        env.set_caller(env.get_account(1));
        let shares = contract.with_tokens(deposit_amount).deposit();

        // Withdraw half
        let half_shares = shares / U512::from(2u64);
        contract.withdraw(half_shares);

        assert_eq!(contract.get_user_shares(env.get_account(1)), half_shares);
    }
}
