use odra::prelude::*;
use odra::casper_types::U256;

/// stCSPR - Liquid Staking Token (CEP-18 Compliant)
///
/// Represents staked CSPR with accrued rewards. This token:
/// - Is fully transferable and composable across DeFi
/// - Appreciates in value as staking rewards accrue (non-rebasing model)
/// - Can be redeemed 1:1 for CSPR through the StakeFlow vault
/// - Implements CEP-18 standard for maximum compatibility
#[odra::module]
pub struct StCSPRToken {
    /// Token metadata
    name: Var<String>,
    symbol: Var<String>,
    decimals: Var<u8>,

    /// Total supply of stCSPR tokens
    total_supply: Var<U256>,

    /// Balances: user address -> token balance
    balances: Mapping<Address, U256>,

    /// Allowances: owner -> spender -> amount
    allowances: Mapping<(Address, Address), U256>,

    /// Vault contract address (only vault can mint/burn)
    vault: Var<Address>,

    /// Exchange rate: CSPR per stCSPR (scaled by 1e9 for precision)
    /// Starts at 1e9 (1:1) and increases as rewards accrue
    exchange_rate: Var<U256>,

    /// Total CSPR backing the stCSPR supply (tracks in vault)
    total_cspr_backing: Var<U256>,

    /// Paused state for emergency controls
    paused: Var<bool>,
}

#[odra::module]
impl StCSPRToken {
    /// Initialize the stCSPR token
    /// When used as SubModule, the parent vault is the caller
    pub fn init(&mut self) {
        let vault_address = self.env().caller();
        self.name.set("Staked CSPR".to_string());
        self.symbol.set("stCSPR".to_string());
        self.decimals.set(9); // Match CSPR decimals
        self.total_supply.set(U256::zero());
        self.vault.set(vault_address);
        self.exchange_rate.set(U256::from(1_000_000_000u64)); // 1:1 initially
        self.total_cspr_backing.set(U256::zero());
        self.paused.set(false);

        self.env().emit_event(TokenInitialized {
            vault: vault_address,
            timestamp: self.env().get_block_time(),
        });
    }

    // ===== CEP-18 STANDARD INTERFACE =====

    /// Get token name
    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    /// Get token symbol
    pub fn symbol(&self) -> String {
        self.symbol.get_or_default()
    }

    /// Get token decimals
    pub fn decimals(&self) -> u8 {
        self.decimals.get_or_default()
    }

    /// Get total supply of stCSPR tokens
    pub fn total_supply(&self) -> U256 {
        self.total_supply.get_or_default()
    }

    /// Get balance of an address
    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get(&owner).unwrap_or(U256::zero())
    }

    /// Transfer tokens from caller to recipient
    pub fn transfer(&mut self, recipient: Address, amount: U256) {
        self.assert_not_paused();
        let caller = self.env().caller();

        assert!(amount > U256::zero(), "Amount must be greater than zero");
        assert!(caller != recipient, "Cannot transfer to self");

        self.transfer_from_to(caller, recipient, amount);

        self.env().emit_event(Transfer {
            from: Some(caller),
            to: Some(recipient),
            amount,
        });
    }

    /// Approve spender to spend tokens on behalf of caller
    pub fn approve(&mut self, spender: Address, amount: U256) {
        self.assert_not_paused();
        let caller = self.env().caller();

        assert!(caller != spender, "Cannot approve self");

        self.allowances.set(&(caller, spender), amount);

        self.env().emit_event(Approval {
            owner: caller,
            spender,
            amount,
        });
    }

    /// Get allowance for spender from owner
    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.get(&(owner, spender)).unwrap_or(U256::zero())
    }

    /// Transfer tokens on behalf of owner (requires approval)
    pub fn transfer_from(&mut self, owner: Address, recipient: Address, amount: U256) {
        self.assert_not_paused();
        let caller = self.env().caller();

        assert!(amount > U256::zero(), "Amount must be greater than zero");
        assert!(owner != recipient, "Cannot transfer to self");

        // Check and update allowance
        let current_allowance = self.allowances.get(&(owner, caller)).unwrap_or(U256::zero());
        assert!(current_allowance >= amount, "Insufficient allowance");

        self.allowances.set(&(owner, caller), current_allowance - amount);

        // Perform transfer
        self.transfer_from_to(owner, recipient, amount);

        self.env().emit_event(Transfer {
            from: Some(owner),
            to: Some(recipient),
            amount,
        });
    }

    // ===== LIQUID STAKING SPECIFIC FUNCTIONS =====

    /// Mint new stCSPR tokens (only vault can call)
    /// Returns the amount of stCSPR minted
    pub fn mint(&mut self, to: Address, cspr_amount: U256) -> U256 {
        self.assert_vault();
        self.assert_not_paused();

        assert!(cspr_amount > U256::zero(), "Amount must be greater than zero");

        // Calculate stCSPR amount based on current exchange rate
        // stCSPR_amount = CSPR_amount * 1e9 / exchange_rate
        let rate = self.exchange_rate.get_or_default();
        let stcspr_amount = (cspr_amount * U256::from(1_000_000_000u64)) / rate;

        // Update balances
        let current_balance = self.balances.get(&to).unwrap_or(U256::zero());
        self.balances.set(&to, current_balance + stcspr_amount);

        // Update total supply
        let total = self.total_supply.get_or_default();
        self.total_supply.set(total + stcspr_amount);

        // Update CSPR backing
        let backing = self.total_cspr_backing.get_or_default();
        self.total_cspr_backing.set(backing + cspr_amount);

        self.env().emit_event(Mint {
            to,
            stcspr_amount,
            cspr_amount,
            exchange_rate: rate,
            timestamp: self.env().get_block_time(),
        });

        self.env().emit_event(Transfer {
            from: None,
            to: Some(to),
            amount: stcspr_amount,
        });

        stcspr_amount
    }

    /// Burn stCSPR tokens (only vault can call)
    /// Returns the amount of CSPR to return to user
    pub fn burn(&mut self, from: Address, stcspr_amount: U256) -> U256 {
        self.assert_vault();
        self.assert_not_paused();

        assert!(stcspr_amount > U256::zero(), "Amount must be greater than zero");

        let balance = self.balances.get(&from).unwrap_or(U256::zero());
        assert!(balance >= stcspr_amount, "Insufficient balance");

        // Calculate CSPR amount based on current exchange rate
        // CSPR_amount = stCSPR_amount * exchange_rate / 1e9
        let rate = self.exchange_rate.get_or_default();
        let cspr_amount = (stcspr_amount * rate) / U256::from(1_000_000_000u64);

        // Update balances
        self.balances.set(&from, balance - stcspr_amount);

        // Update total supply
        let total = self.total_supply.get_or_default();
        self.total_supply.set(total - stcspr_amount);

        // Update CSPR backing
        let backing = self.total_cspr_backing.get_or_default();
        self.total_cspr_backing.set(backing - cspr_amount);

        self.env().emit_event(Burn {
            from,
            stcspr_amount,
            cspr_amount,
            exchange_rate: rate,
            timestamp: self.env().get_block_time(),
        });

        self.env().emit_event(Transfer {
            from: Some(from),
            to: None,
            amount: stcspr_amount,
        });

        cspr_amount
    }

    /// Update exchange rate when rewards are compounded
    /// Only vault can call this
    pub fn update_exchange_rate(&mut self, new_total_cspr: U256) {
        self.assert_vault();

        let total_stcspr = self.total_supply.get_or_default();

        if total_stcspr == U256::zero() {
            // No tokens minted yet, keep rate at 1:1
            return;
        }

        // New exchange rate = total_cspr * 1e9 / total_stcspr
        let new_rate = (new_total_cspr * U256::from(1_000_000_000u64)) / total_stcspr;
        let old_rate = self.exchange_rate.get_or_default();

        self.exchange_rate.set(new_rate);
        self.total_cspr_backing.set(new_total_cspr);

        self.env().emit_event(ExchangeRateUpdated {
            old_rate,
            new_rate,
            total_cspr: new_total_cspr,
            total_stcspr,
            timestamp: self.env().get_block_time(),
        });
    }

    /// Get current exchange rate (CSPR per stCSPR, scaled by 1e9)
    pub fn get_exchange_rate(&self) -> U256 {
        self.exchange_rate.get_or_default()
    }

    /// Get CSPR value for a given amount of stCSPR
    pub fn stcspr_to_cspr(&self, stcspr_amount: U256) -> U256 {
        let rate = self.exchange_rate.get_or_default();
        (stcspr_amount * rate) / U256::from(1_000_000_000u64)
    }

    /// Get stCSPR amount for a given CSPR value
    pub fn cspr_to_stcspr(&self, cspr_amount: U256) -> U256 {
        let rate = self.exchange_rate.get_or_default();
        (cspr_amount * U256::from(1_000_000_000u64)) / rate
    }

    /// Get total CSPR backing all stCSPR tokens
    pub fn get_total_cspr_backing(&self) -> U256 {
        self.total_cspr_backing.get_or_default()
    }

    /// Get vault address
    pub fn get_vault(&self) -> Address {
        self.vault.get().expect("Vault not set")
    }

    // ===== ADMIN FUNCTIONS =====

    /// Emergency pause (only vault can call)
    pub fn pause(&mut self) {
        self.assert_vault();
        self.paused.set(true);

        self.env().emit_event(Paused {
            timestamp: self.env().get_block_time(),
        });
    }

    /// Unpause (only vault can call)
    pub fn unpause(&mut self) {
        self.assert_vault();
        self.paused.set(false);

        self.env().emit_event(Unpaused {
            timestamp: self.env().get_block_time(),
        });
    }

    /// Check if token is paused
    pub fn is_paused(&self) -> bool {
        self.paused.get_or_default()
    }

    // ===== INTERNAL FUNCTIONS =====

    /// Internal transfer function
    fn transfer_from_to(&mut self, from: Address, to: Address, amount: U256) {
        let from_balance = self.balances.get(&from).unwrap_or(U256::zero());
        assert!(from_balance >= amount, "Insufficient balance");

        let to_balance = self.balances.get(&to).unwrap_or(U256::zero());

        self.balances.set(&from, from_balance - amount);
        self.balances.set(&to, to_balance + amount);
    }

    /// Assert caller is vault
    fn assert_vault(&self) {
        let caller = self.env().caller();
        let vault = self.vault.get().expect("Vault not set");
        assert!(caller == vault, "Only vault can call this function");
    }

    /// Assert token is not paused
    fn assert_not_paused(&self) {
        assert!(!self.paused.get_or_default(), "Token is paused");
    }
}

// ===== EVENTS =====

#[odra::event]
pub struct TokenInitialized {
    pub vault: Address,
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
pub struct Mint {
    pub to: Address,
    pub stcspr_amount: U256,
    pub cspr_amount: U256,
    pub exchange_rate: U256,
    pub timestamp: u64,
}

#[odra::event]
pub struct Burn {
    pub from: Address,
    pub stcspr_amount: U256,
    pub cspr_amount: U256,
    pub exchange_rate: U256,
    pub timestamp: u64,
}

#[odra::event]
pub struct ExchangeRateUpdated {
    pub old_rate: U256,
    pub new_rate: U256,
    pub total_cspr: U256,
    pub total_stcspr: U256,
    pub timestamp: u64,
}

#[odra::event]
pub struct Paused {
    pub timestamp: u64,
}

#[odra::event]
pub struct Unpaused {
    pub timestamp: u64,
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::Deployer;

    #[test]
    fn test_initialization() {
        let env = odra_test::env();
        use odra::host::NoArgs;
        let token = StCSPRToken::deploy(&env, NoArgs);

        assert_eq!(token.name(), "Staked CSPR");
        assert_eq!(token.symbol(), "stCSPR");
        assert_eq!(token.decimals(), 9);
        assert_eq!(token.total_supply(), U256::zero());
        assert_eq!(token.get_exchange_rate(), U256::from(1_000_000_000u64));
    }

    #[test]
    fn test_mint() {
        let env = odra_test::env();
        use odra::host::NoArgs;
        let mut token = StCSPRToken::deploy(&env, NoArgs);

        let user = env.get_account(1);
        let cspr_amount = U256::from(100_000_000_000u64); // 100 CSPR

        // Mint as vault (vault is the deployer/caller)
        let stcspr_amount = token.mint(user, cspr_amount);

        assert_eq!(stcspr_amount, cspr_amount); // 1:1 initially
        assert_eq!(token.balance_of(user), stcspr_amount);
        assert_eq!(token.total_supply(), stcspr_amount);
    }

    #[test]
    fn test_burn() {
        let env = odra_test::env();
        use odra::host::NoArgs;
        let mut token = StCSPRToken::deploy(&env, NoArgs);

        let user = env.get_account(1);
        let cspr_amount = U256::from(100_000_000_000u64);

        let stcspr_amount = token.mint(user, cspr_amount);

        // Burn half
        let burn_amount = stcspr_amount / U256::from(2u64);
        let cspr_returned = token.burn(user, burn_amount);

        assert_eq!(token.balance_of(user), stcspr_amount - burn_amount);
        assert_eq!(cspr_returned, cspr_amount / U256::from(2u64));
    }

    #[test]
    fn test_transfer() {
        let env = odra_test::env();
        use odra::host::NoArgs;
        let mut token = StCSPRToken::deploy(&env, NoArgs);

        let user1 = env.get_account(1);
        let user2 = env.get_account(2);
        let amount = U256::from(100_000_000_000u64);

        // Mint to user1 (deployer is vault)
        token.mint(user1, amount);

        // Transfer from user1 to user2
        env.set_caller(user1);
        let transfer_amount = amount / U256::from(2u64);
        token.transfer(user2, transfer_amount);

        assert_eq!(token.balance_of(user1), amount - transfer_amount);
        assert_eq!(token.balance_of(user2), transfer_amount);
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let env = odra_test::env();
        use odra::host::NoArgs;
        let mut token = StCSPRToken::deploy(&env, NoArgs);

        let owner = env.get_account(1);
        let spender = env.get_account(2);
        let recipient = env.get_account(3);
        let amount = U256::from(100_000_000_000u64);

        // Mint to owner (deployer is vault)
        token.mint(owner, amount);

        // Owner approves spender
        env.set_caller(owner);
        token.approve(spender, amount);
        assert_eq!(token.allowance(owner, spender), amount);

        // Spender transfers on behalf of owner
        env.set_caller(spender);
        let transfer_amount = amount / U256::from(2u64);
        token.transfer_from(owner, recipient, transfer_amount);

        assert_eq!(token.balance_of(recipient), transfer_amount);
        assert_eq!(token.allowance(owner, spender), amount - transfer_amount);
    }

    #[test]
    fn test_exchange_rate_update() {
        let env = odra_test::env();
        use odra::host::NoArgs;
        let mut token = StCSPRToken::deploy(&env, NoArgs);

        let user = env.get_account(1);
        let initial_cspr = U256::from(100_000_000_000u64); // 100 CSPR

        let stcspr_minted = token.mint(user, initial_cspr);

        // Simulate rewards: now 110 CSPR backs the same stCSPR
        let new_total_cspr = U256::from(110_000_000_000u64);
        token.update_exchange_rate(new_total_cspr);

        // Exchange rate should have increased
        let new_rate = token.get_exchange_rate();
        assert!(new_rate > U256::from(1_000_000_000u64));

        // User's stCSPR is worth more CSPR now
        let cspr_value = token.stcspr_to_cspr(stcspr_minted);
        assert_eq!(cspr_value, new_total_cspr);
    }

    #[test]
    fn test_conversion_functions() {
        let env = odra_test::env();
        use odra::host::NoArgs;
        let token = StCSPRToken::deploy(&env, NoArgs);

        let cspr = U256::from(100_000_000_000u64);
        let stcspr = token.cspr_to_stcspr(cspr);
        let cspr_back = token.stcspr_to_cspr(stcspr);

        assert_eq!(cspr, cspr_back); // Round-trip conversion
    }

    #[test]
    fn test_pause_and_unpause() {
        let env = odra_test::env();
        use odra::host::NoArgs;
        let mut token = StCSPRToken::deploy(&env, NoArgs);

        // Deployer is vault
        token.pause();
        assert!(token.is_paused());

        token.unpause();
        assert!(!token.is_paused());
    }
}
