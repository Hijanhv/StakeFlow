//! StakeFlow Testnet Deployment Script
//! 
//! Deploys StCSPRToken, StakeFlowVaultV3, and StakeFlowGovernance
//! with proper initialization to Casper testnet.

use stakeflow::stcspr_token::StCSPRToken;
use stakeflow::stakeflow_vault_v3::StakeFlowVaultV3;
use stakeflow::governance::StakeFlowGovernance;
use odra::host::{HostEnv, NoArgs};
use odra::casper_types::U512;
use odra_cli::{
    deploy::DeployScript,
    ContractProvider, DeployedContractsContainer, DeployerExt, OdraCli,
};

/// Deploy StCSPR Token
pub struct TokenDeployScript;

impl DeployScript for TokenDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer
    ) -> Result<(), odra_cli::deploy::Error> {
        println!("📦 Deploying StCSPR Token...");
        
        let _token = StCSPRToken::load_or_deploy(
            env,
            NoArgs,
            container,
            400_000_000_000 // 400 CSPR gas limit
        )?;

        println!("✅ StCSPR Token deployed successfully!");
        Ok(())
    }
}

/// Deploy StakeFlow Vault V3
pub struct VaultDeployScript;

impl DeployScript for VaultDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer
    ) -> Result<(), odra_cli::deploy::Error> {
        println!("📦 Deploying StakeFlow Vault V3...");
        
        // Treasury address (using deployer address for now)
        let treasury = env.caller();
        let unbonding_days = 7u64; // 7-day unbonding period
        
        let _vault = StakeFlowVaultV3::load_or_deploy(
            env,
            (treasury, unbonding_days),
            container,
            400_000_000_000 // 400 CSPR gas limit
        )?;

        println!("✅ StakeFlow Vault V3 deployed successfully!");
        Ok(())
    }
}

/// Deploy StakeFlow Governance
pub struct GovernanceDeployScript;

impl DeployScript for GovernanceDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer
    ) -> Result<(), odra_cli::deploy::Error> {
        println!("📦 Deploying StakeFlow Governance...");
        
        let _governance = StakeFlowGovernance::load_or_deploy(
            env,
            NoArgs,
            container,
            400_000_000_000 // 400 CSPR gas limit
        )?;

        println!("✅ StakeFlow Governance deployed successfully!");
        Ok(())
    }
}

/// Main deployment orchestrator
pub fn main() {
    println!("🚀 StakeFlow Testnet Deployment\n");
    
    OdraCli::new()
        .about("Deploy StakeFlow contracts to Casper testnet")
        .deploy(TokenDeployScript)
        .contract::<StCSPRToken>()
        .deploy(VaultDeployScript)
        .contract::<StakeFlowVaultV3>()
        .deploy(GovernanceDeployScript)
        .contract::<StakeFlowGovernance>()
        .build()
        .run();
    
    println!("\n✨ All contracts deployed successfully!");
    println!("📝 Check contract addresses in the output above");
}
