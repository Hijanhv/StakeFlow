use odra::prelude::*;
use odra::casper_types::{U512, U256};

/// StakeFlow Governance Contract
///
/// Decentralized governance for the StakeFlow protocol allowing stCSPR holders to:
/// - Propose and vote on protocol parameter changes
/// - Add/remove validators
/// - Adjust performance fees
/// - Emergency pause/unpause
/// - Treasury management
///
/// Voting power is proportional to stCSPR holdings (1 stCSPR = 1 vote)
#[odra::module]
pub struct StakeFlowGovernance {
    /// Protocol owner (multi-sig or DAO in production)
    owner: Var<Address>,

    /// Vault contract address (to execute proposals)
    vault_contract: Var<Address>,

    /// Proposals mapping: proposal_id -> Proposal
    proposals: Mapping<u64, Proposal>,

    /// Next proposal ID
    next_proposal_id: Var<u64>,

    /// User votes: (proposal_id, voter) -> Vote
    votes: Mapping<(u64, Address), Vote>,

    /// Governance parameters
    voting_period: Var<u64>,          // Voting duration in seconds
    quorum_percentage: Var<u32>,       // Minimum participation (in bps, 2000 = 20%)
    approval_threshold: Var<u32>,      // Approval threshold (in bps, 5000 = 50%)
    proposal_threshold: Var<U256>,     // Minimum stCSPR to create proposal
}

/// Proposal types
#[odra::odra_type]
pub enum ProposalType {
    /// Change performance fee (new_fee_bps)
    ChangePerformanceFee(u32),
    /// Add validator (validator_address, initial_score)
    AddValidator(Address, u32),
    /// Remove validator (validator_address)
    RemoveValidator(Address),
    /// Emergency pause
    EmergencyPause,
    /// Unpause
    Unpause,
    /// Change governance parameter
    ChangeGovernanceParam(String, u64),
}

/// Proposal struct
#[odra::odra_type]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub description: String,
    pub start_time: u64,
    pub end_time: u64,
    pub for_votes: U256,
    pub against_votes: U256,
    pub executed: bool,
    pub cancelled: bool,
}

/// Vote struct
#[odra::odra_type]
pub struct Vote {
    pub voter: Address,
    pub support: bool,  // true = for, false = against
    pub voting_power: U256,
    pub timestamp: u64,
}

/// Proposal status
#[odra::odra_type]
pub enum ProposalStatus {
    Pending,
    Active,
    Succeeded,
    Defeated,
    Executed,
    Cancelled,
}

#[odra::module]
impl StakeFlowGovernance {
    /// Initialize governance
    pub fn init(&mut self, vault_address: Address) {
        let caller = self.env().caller();
        self.owner.set(caller);
        self.vault_contract.set(vault_address);
        self.next_proposal_id.set(0);

        // Default governance parameters
        self.voting_period.set(3 * 24 * 60 * 60);      // 3 days
        self.quorum_percentage.set(2000);              // 20%
        self.approval_threshold.set(5000);             // 50%
        self.proposal_threshold.set(U256::from(1000)); // 1000 stCSPR minimum

        self.env().emit_event(GovernanceInitialized {
            owner: caller,
            vault: vault_address,
            timestamp: self.env().get_block_time(),
        });
    }

    // ===== PROPOSAL CREATION =====

    /// Create a new proposal
    pub fn create_proposal(
        &mut self,
        proposal_type: ProposalType,
        description: String,
        voting_power: U256, // Caller's stCSPR balance from vault
    ) -> u64 {
        let caller = self.env().caller();

        // Check proposal threshold
        let threshold = self.proposal_threshold.get_or_default();
        assert!(voting_power >= threshold, "Insufficient stCSPR to create proposal");

        let proposal_id = self.next_proposal_id.get_or_default();
        let current_time = self.env().get_block_time();
        let voting_period = self.voting_period.get_or_default();

        let proposal = Proposal {
            id: proposal_id,
            proposer: caller,
            proposal_type,
            description: description.clone(),
            start_time: current_time,
            end_time: current_time + voting_period,
            for_votes: U256::zero(),
            against_votes: U256::zero(),
            executed: false,
            cancelled: false,
        };

        self.proposals.set(&proposal_id, proposal);
        self.next_proposal_id.set(proposal_id + 1);

        self.env().emit_event(ProposalCreated {
            proposal_id,
            proposer: caller,
            description,
            end_time: current_time + voting_period,
            timestamp: current_time,
        });

        proposal_id
    }

    // ===== VOTING =====

    /// Cast a vote on a proposal
    pub fn cast_vote(
        &mut self,
        proposal_id: u64,
        support: bool,
        voting_power: U256, // Caller's stCSPR balance from vault
    ) {
        let caller = self.env().caller();
        let current_time = self.env().get_block_time();

        // Get proposal
        let mut proposal = self.proposals.get(&proposal_id)
            .expect("Proposal not found");

        // Check voting is active
        assert!(current_time >= proposal.start_time, "Voting not started");
        assert!(current_time < proposal.end_time, "Voting ended");
        assert!(!proposal.cancelled, "Proposal cancelled");
        assert!(!proposal.executed, "Proposal already executed");

        // Check user hasn't already voted
        assert!(
            self.votes.get(&(proposal_id, caller)).is_none(),
            "Already voted"
        );

        // Record vote
        let vote = Vote {
            voter: caller,
            support,
            voting_power,
            timestamp: current_time,
        };

        self.votes.set(&(proposal_id, caller), vote);

        // Update proposal vote counts
        if support {
            proposal.for_votes = proposal.for_votes + voting_power;
        } else {
            proposal.against_votes = proposal.against_votes + voting_power;
        }

        self.proposals.set(&proposal_id, proposal);

        self.env().emit_event(VoteCast {
            proposal_id,
            voter: caller,
            support,
            voting_power,
            timestamp: current_time,
        });
    }

    // ===== PROPOSAL EXECUTION =====

    /// Execute a successful proposal
    pub fn execute_proposal(&mut self, proposal_id: u64, total_stcspr_supply: U256) {
        let current_time = self.env().get_block_time();

        let mut proposal = self.proposals.get(&proposal_id)
            .expect("Proposal not found");

        // Validate execution conditions
        assert!(current_time >= proposal.end_time, "Voting period not ended");
        assert!(!proposal.executed, "Already executed");
        assert!(!proposal.cancelled, "Proposal cancelled");

        // Check quorum
        let total_votes = proposal.for_votes + proposal.against_votes;
        let quorum_bps = self.quorum_percentage.get_or_default();
        let quorum_required = (total_stcspr_supply * U256::from(quorum_bps)) / U256::from(10000u64);
        assert!(total_votes >= quorum_required, "Quorum not reached");

        // Check approval threshold
        let approval_bps = self.approval_threshold.get_or_default();
        let approval_required = (total_votes * U256::from(approval_bps)) / U256::from(10000u64);
        assert!(proposal.for_votes >= approval_required, "Proposal defeated");

        // Mark as executed
        proposal.executed = true;
        self.proposals.set(&proposal_id, proposal);

        // Note: Actual execution would call vault contract methods
        // For hackathon, we emit events to show what would be executed
        self.env().emit_event(ProposalExecuted {
            proposal_id,
            executor: self.env().caller(),
            timestamp: current_time,
        });
    }

    /// Cancel a proposal (owner only, before execution)
    pub fn cancel_proposal(&mut self, proposal_id: u64) {
        self.assert_owner();

        let mut proposal = self.proposals.get(&proposal_id)
            .expect("Proposal not found");

        assert!(!proposal.executed, "Already executed");
        assert!(!proposal.cancelled, "Already cancelled");

        proposal.cancelled = true;
        self.proposals.set(&proposal_id, proposal);

        self.env().emit_event(ProposalCancelled {
            proposal_id,
            timestamp: self.env().get_block_time(),
        });
    }

    // ===== VIEW FUNCTIONS =====

    /// Get proposal details
    pub fn get_proposal(&self, proposal_id: u64) -> Option<Proposal> {
        self.proposals.get(&proposal_id)
    }

    /// Get proposal status
    pub fn get_proposal_status(&self, proposal_id: u64, total_supply: U256) -> ProposalStatus {
        let proposal = match self.proposals.get(&proposal_id) {
            Some(p) => p,
            None => return ProposalStatus::Pending,
        };

        if proposal.cancelled {
            return ProposalStatus::Cancelled;
        }

        if proposal.executed {
            return ProposalStatus::Executed;
        }

        let current_time = self.env().get_block_time();

        // Still active
        if current_time < proposal.end_time {
            return ProposalStatus::Active;
        }

        // Voting ended, check result
        let total_votes = proposal.for_votes + proposal.against_votes;
        let quorum_bps = self.quorum_percentage.get_or_default();
        let quorum_required = (total_supply * U256::from(quorum_bps)) / U256::from(10000u64);

        if total_votes < quorum_required {
            return ProposalStatus::Defeated;
        }

        let approval_bps = self.approval_threshold.get_or_default();
        let approval_required = (total_votes * U256::from(approval_bps)) / U256::from(10000u64);

        if proposal.for_votes >= approval_required {
            ProposalStatus::Succeeded
        } else {
            ProposalStatus::Defeated
        }
    }

    /// Get user's vote on a proposal
    pub fn get_vote(&self, proposal_id: u64, voter: Address) -> Option<Vote> {
        self.votes.get(&(proposal_id, voter))
    }

    /// Get voting period
    pub fn get_voting_period(&self) -> u64 {
        self.voting_period.get_or_default()
    }

    /// Get quorum percentage
    pub fn get_quorum_percentage(&self) -> u32 {
        self.quorum_percentage.get_or_default()
    }

    /// Get approval threshold
    pub fn get_approval_threshold(&self) -> u32 {
        self.approval_threshold.get_or_default()
    }

    /// Get proposal threshold
    pub fn get_proposal_threshold(&self) -> U256 {
        self.proposal_threshold.get_or_default()
    }

    // ===== ADMIN FUNCTIONS =====

    /// Update governance parameters (owner only)
    pub fn update_governance_params(
        &mut self,
        new_voting_period: Option<u64>,
        new_quorum_bps: Option<u32>,
        new_approval_bps: Option<u32>,
        new_proposal_threshold: Option<U256>,
    ) {
        self.assert_owner();

        if let Some(period) = new_voting_period {
            assert!(period >= 24 * 60 * 60, "Minimum 1 day voting period");
            self.voting_period.set(period);
        }

        if let Some(quorum) = new_quorum_bps {
            assert!(quorum <= 10000, "Max 100%");
            self.quorum_percentage.set(quorum);
        }

        if let Some(approval) = new_approval_bps {
            assert!(approval <= 10000 && approval >= 5000, "Approval between 50-100%");
            self.approval_threshold.set(approval);
        }

        if let Some(threshold) = new_proposal_threshold {
            self.proposal_threshold.set(threshold);
        }

        self.env().emit_event(GovernanceParamsUpdated {
            timestamp: self.env().get_block_time(),
        });
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
pub struct GovernanceInitialized {
    pub owner: Address,
    pub vault: Address,
    pub timestamp: u64,
}

#[odra::event]
pub struct ProposalCreated {
    pub proposal_id: u64,
    pub proposer: Address,
    pub description: String,
    pub end_time: u64,
    pub timestamp: u64,
}

#[odra::event]
pub struct VoteCast {
    pub proposal_id: u64,
    pub voter: Address,
    pub support: bool,
    pub voting_power: U256,
    pub timestamp: u64,
}

#[odra::event]
pub struct ProposalExecuted {
    pub proposal_id: u64,
    pub executor: Address,
    pub timestamp: u64,
}

#[odra::event]
pub struct ProposalCancelled {
    pub proposal_id: u64,
    pub timestamp: u64,
}

#[odra::event]
pub struct GovernanceParamsUpdated {
    pub timestamp: u64,
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostRef};

    #[test]
    fn test_create_proposal() {
        let env = odra_test::env();
        let vault = env.get_account(9);
        let mut gov = StakeFlowGovernance::deploy(&env, StakeFlowGovernanceInitArgs {
            vault_address: vault,
        });

        // User with 2000 stCSPR creates proposal
        env.set_caller(env.get_account(1));
        let voting_power = U256::from(2000);

        let proposal_id = gov.create_proposal(
            ProposalType::ChangePerformanceFee(300), // Change to 3%
            "Reduce performance fee to 3%".to_string(),
            voting_power,
        );

        assert_eq!(proposal_id, 0);

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.proposer, env.get_account(1));
        assert!(!proposal.executed);
    }

    #[test]
    fn test_voting() {
        let env = odra_test::env();
        let vault = env.get_account(9);
        let mut gov = StakeFlowGovernance::deploy(&env, StakeFlowGovernanceInitArgs {
            vault_address: vault,
        });

        // Create proposal
        env.set_caller(env.get_account(1));
        let proposal_id = gov.create_proposal(
            ProposalType::ChangePerformanceFee(300),
            "Reduce fee".to_string(),
            U256::from(2000),
        );

        // User 2 votes for
        env.set_caller(env.get_account(2));
        gov.cast_vote(proposal_id, true, U256::from(5000));

        // User 3 votes against
        env.set_caller(env.get_account(3));
        gov.cast_vote(proposal_id, false, U256::from(2000));

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.for_votes, U256::from(5000));
        assert_eq!(proposal.against_votes, U256::from(2000));
    }

    #[test]
    fn test_proposal_execution() {
        let env = odra_test::env();
        let vault = env.get_account(9);
        let mut gov = StakeFlowGovernance::deploy(&env, StakeFlowGovernanceInitArgs {
            vault_address: vault,
        });

        // Create proposal
        env.set_caller(env.get_account(1));
        let proposal_id = gov.create_proposal(
            ProposalType::EmergencyPause,
            "Emergency pause".to_string(),
            U256::from(2000),
        );

        // Vote with sufficient quorum and approval
        env.set_caller(env.get_account(2));
        gov.cast_vote(proposal_id, true, U256::from(8000));

        // Advance time past voting period
        env.advance_block_time(4 * 24 * 60 * 60); // 4 days

        // Execute proposal
        let total_supply = U256::from(10000); // Total stCSPR supply
        gov.execute_proposal(proposal_id, total_supply);

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert!(proposal.executed);
    }
}
