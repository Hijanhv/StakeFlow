use odra::prelude::*;
use odra::casper_types::U512;

/// Minimal StakeFlow contract optimized for low gas deployment
#[odra::module]
pub struct StakeFlowMinimal {
    total_deposits: Var<U512>,
    user_deposits: Mapping<Address, U512>,
    contract_owner: Var<Address>,
}

#[odra::module]
impl StakeFlowMinimal {
    pub fn init(&mut self) {
        self.total_deposits.set(U512::zero());
        self.contract_owner.set(self.env().caller());
    }

    /// Deposit CSPR and receive stCSPR (1:1 ratio for minimal version)
    #[odra(payable)]
    pub fn deposit(&mut self) {
        let caller = self.env().caller();
        let amount = self.env().attached_value();
        
        assert!(amount > U512::zero(), "Amount must be greater than zero");
        
        let current = self.user_deposits.get(&caller).unwrap_or(U512::zero());
        self.user_deposits.set(&caller, current + amount);
        
        let total = self.total_deposits.get_or_default();
        self.total_deposits.set(total + amount);
    }

    /// Withdraw CSPR by burning stCSPR
    pub fn withdraw(&mut self, amount: U512) {
        let caller = self.env().caller();
        let balance = self.user_deposits.get(&caller).unwrap_or(U512::zero());
        
        assert!(balance >= amount, "Insufficient balance");
        
        self.user_deposits.set(&caller, balance - amount);
        
        let total = self.total_deposits.get_or_default();
        self.total_deposits.set(total - amount);
        
        self.env().transfer_tokens(&caller, &amount);
    }

    /// Get user's stCSPR balance
    pub fn get_balance(&self, user: Address) -> U512 {
        self.user_deposits.get(&user).unwrap_or(U512::zero())
    }

    /// Get total value locked
    pub fn get_total_deposits(&self) -> U512 {
        self.total_deposits.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostRef};

    #[test]
    fn test_minimal_deposit() {
        let env = odra_test::env();
        use odra::host::NoArgs;
        let mut contract = StakeFlowMinimal::deploy(&env, NoArgs);
        
        let amount = U512::from(1000);
        contract.with_tokens(amount).deposit();
        
        assert_eq!(contract.get_total_deposits(), amount);
    }
}
