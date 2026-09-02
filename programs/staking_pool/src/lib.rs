#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

//! Milestone 1: scaffold for the Anchor staking program.

pub mod constants;
pub mod error;
pub mod math;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    constants::{ADMIN_COUNT, DEVNET_MAX_REWARD_RATE_PER_SLOT, TOKEN_DECIMALS},
    error::StakingError,
    state::{Pool, POOL_AUTHORITY_SEED, POOL_SEED, STATE_VERSION},
};

declare_id!("Fg6PaFpoGXkYsidMpWxTWqkFrnDRBTTnyW6m9n6eGJZ");

pub const PROGRAM_NAME: &str = "staking_pool";
pub const PROGRAM_VERSION: u8 = 1;

#[program]
pub mod staking_pool {
    use super::*;

    /// Milestone 7: create the canonical pool account and PDA-owned vault ATAs.
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        pool_id: u64,
        admins: [Pubkey; ADMIN_COUNT],
        max_reward_rate_per_slot: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.stake_mint.decimals == TOKEN_DECIMALS,
            StakingError::InvalidTokenDecimals
        );
        require!(
            ctx.accounts.reward_mint.decimals == TOKEN_DECIMALS,
            StakingError::InvalidTokenDecimals
        );
        require_keys_neq!(
            ctx.accounts.stake_mint.key(),
            ctx.accounts.reward_mint.key(),
            StakingError::InvalidMintPair
        );
        require_distinct_admins(&admins)?;
        require!(
            max_reward_rate_per_slot <= DEVNET_MAX_REWARD_RATE_PER_SLOT,
            StakingError::RewardRateAboveMaximum
        );
        // Only allow normal SPL tokens for this staking pool.
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            anchor_spl::token::ID,
            StakingError::InvalidTokenProgram
        );

        let pool = &mut ctx.accounts.pool;
        pool.version = STATE_VERSION;
        pool.initializer = ctx.accounts.initializer.key();
        pool.pool_id = pool_id;
        pool.pool_bump = ctx.bumps.pool;
        pool.pool_authority_bump = ctx.bumps.pool_authority;
        pool.stake_mint = ctx.accounts.stake_mint.key();
        pool.reward_mint = ctx.accounts.reward_mint.key();
        pool.stake_vault = ctx.accounts.stake_vault.key();
        pool.reward_vault = ctx.accounts.reward_vault.key();
        pool.admins = admins;
        pool.admin_epoch = 0;
        pool.next_proposal_id = 0;
        pool.paused = true;
        pool.max_reward_rate_per_slot = max_reward_rate_per_slot;
        pool.reward_rate_per_slot = 0;
        pool.last_update_slot = Clock::get()?.slot;
        pool.total_staked = 0;
        pool.acc_reward_per_stake_scaled = 0;
        pool.remaining_reward_budget_scaled = 0;
        pool.allocated_liability_scaled = 0;

        emit!(PoolInitialized {
            pool: pool.key(),
            initializer: pool.initializer,
            pool_id,
            stake_mint: pool.stake_mint,
            reward_mint: pool.reward_mint,
            stake_vault: pool.stake_vault,
            reward_vault: pool.reward_vault,
            max_reward_rate_per_slot,
            slot: pool.last_update_slot,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        init,
        payer = initializer,
        space = Pool::SPACE,
        seeds = [POOL_SEED, initializer.key().as_ref(), &pool_id.to_le_bytes()],
        bump
    )]
    pub pool: Account<'info, Pool>,
    /// CHECK: Milestone 7: this PDA has no data account; Anchor validates its
    /// canonical seeds here and it only acts as the vault ATA authority.
    #[account(seeds = [POOL_AUTHORITY_SEED, pool.key().as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,
    pub stake_mint: Account<'info, Mint>,
    pub reward_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint = stake_mint,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program
    )]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint = reward_mint,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program
    )]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[event]
pub struct PoolInitialized {
    pub pool: Pubkey,
    pub initializer: Pubkey,
    pub pool_id: u64,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub stake_vault: Pubkey,
    pub reward_vault: Pubkey,
    pub max_reward_rate_per_slot: u64,
    pub slot: u64,
}

fn require_distinct_admins(admins: &[Pubkey; ADMIN_COUNT]) -> Result<()> {
    for (index, admin) in admins.iter().enumerate() {
        require_keys_neq!(*admin, Pubkey::default(), StakingError::InvalidAdminSet);
        for previous_admin in admins.iter().take(index) {
            require_keys_neq!(*admin, *previous_admin, StakingError::InvalidAdminSet);
        }
    }

    Ok(())
}

/// Milestone 2: harmless baseline check used before protocol behavior exists.
pub fn program_identity() -> (&'static str, u8) {
    (PROGRAM_NAME, PROGRAM_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staking_program_identity_is_stable() {
        assert_eq!(program_identity(), ("staking_pool", 1));
    }
}
