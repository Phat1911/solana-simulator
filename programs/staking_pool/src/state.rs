//! Milestone 6: fixed-size account schemas and canonical PDA recipes.

use anchor_lang::prelude::*;

use crate::constants::ADMIN_COUNT;

pub const STATE_VERSION: u8 = 1;
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

pub const POOL_SEED: &[u8] = b"pool";
pub const POOL_AUTHORITY_SEED: &[u8] = b"pool-authority";
pub const POSITION_SEED: &[u8] = b"position";
pub const PROPOSAL_SEED: &[u8] = b"proposal";

#[account]
#[derive(Debug, PartialEq, Eq, InitSpace)]
pub struct Pool {
    pub version: u8,
    pub initializer: Pubkey,
    pub pool_id: u64,
    pub pool_bump: u8,
    pub pool_authority_bump: u8,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub stake_vault: Pubkey,
    pub reward_vault: Pubkey,
    pub admins: [Pubkey; ADMIN_COUNT],
    pub admin_epoch: u64,
    pub next_proposal_id: u64,
    pub paused: bool,
    pub max_reward_rate_per_slot: u64,
    pub reward_rate_per_slot: u64,
    pub last_update_slot: u64,
    pub total_staked: u64,
    pub acc_reward_per_stake_scaled: u128,
    pub remaining_reward_budget_scaled: u128,
    pub allocated_liability_scaled: u128,
}

impl Pool {
    pub const LEN: usize = <Self as Space>::INIT_SPACE;
    pub const SPACE: usize = ANCHOR_DISCRIMINATOR_SIZE + Self::LEN;
}

#[account]
#[derive(Debug, PartialEq, Eq, InitSpace)]
pub struct Position {
    pub version: u8,
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub bump: u8,
    pub staked_amount: u64,
    pub reward_debt_scaled: u128,
    pub pending_reward_scaled: u128,
}

impl Position {
    pub const LEN: usize = <Self as Space>::INIT_SPACE;
    pub const SPACE: usize = ANCHOR_DISCRIMINATOR_SIZE + Self::LEN;
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum ProposalAction {
    SetRewardRate {
        new_rate: u64,
    },
    UnpausePool,
    ReplaceAdmin {
        old_admin: Pubkey,
        new_admin: Pubkey,
    },
}

impl ProposalAction {
    pub const MAX_SIZE: usize = <Self as Space>::INIT_SPACE;
}

#[account]
#[derive(Debug, PartialEq, Eq, InitSpace)]
pub struct Proposal {
    pub version: u8,
    pub pool: Pubkey,
    pub proposal_id: u64,
    pub creator: Pubkey,
    pub admin_epoch: u64,
    pub action: ProposalAction,
    pub approvals: [bool; ADMIN_COUNT],
    pub approval_count: u8,
    pub created_slot: u64,
    pub expires_at_slot: u64,
    pub executed: bool,
    pub bump: u8,
}

impl Proposal {
    pub const LEN: usize = <Self as Space>::INIT_SPACE;
    pub const SPACE: usize = ANCHOR_DISCRIMINATOR_SIZE + Self::LEN;
}

pub fn derive_pool_pda(program_id: &Pubkey, initializer: &Pubkey, pool_id: u64) -> (Pubkey, u8) {
    let pool_id_bytes = pool_id.to_le_bytes();
    Pubkey::find_program_address(
        &[POOL_SEED, initializer.as_ref(), pool_id_bytes.as_ref()],
        program_id,
    )
}

pub fn derive_pool_authority_pda(program_id: &Pubkey, pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[POOL_AUTHORITY_SEED, pool.as_ref()], program_id)
}

pub fn derive_position_pda(program_id: &Pubkey, pool: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[POSITION_SEED, pool.as_ref(), user.as_ref()], program_id)
}

pub fn derive_proposal_pda(program_id: &Pubkey, pool: &Pubkey, proposal_id: u64) -> (Pubkey, u8) {
    let proposal_id_bytes = proposal_id.to_le_bytes();
    Pubkey::find_program_address(
        &[PROPOSAL_SEED, pool.as_ref(), proposal_id_bytes.as_ref()],
        program_id,
    )
}
