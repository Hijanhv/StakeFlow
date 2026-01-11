#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;

pub mod flipper;
pub mod stakeflow_vault;
pub mod stakeflow_minimal;
pub mod stcspr_token;
// pub mod stakeflow_vault_v2;  // DISABLED - using V3 instead
pub mod stakeflow_vault_v3;
pub mod governance;
