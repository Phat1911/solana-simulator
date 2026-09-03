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
    token::{self, Mint, Token, TokenAccount, TransferChecked},
};

use crate::{
    constants::{
        ADMIN_COUNT, ADMIN_THRESHOLD, DEVNET_MAX_REWARD_RATE_PER_SLOT, PROPOSAL_TTL_SLOTS,
        TOKEN_DECIMALS,
    },
    error::StakingError,
    math::{
        checked_add_u64, checked_claimable_base_units, checked_paid_scaled,
        checked_scale_base_units, checked_sub_scaled, checked_sub_u64, checkpoint_pool_rewards,
        claim_position_rewards, forfeit_position_rewards, require_positive_amount,
        reset_position_reward_debt, settle_position_rewards, ClaimPositionInput,
        ForfeitPositionInput, PoolCheckpointInput, PositionSettlementInput,
    },
    state::{
        Pool, Position, Proposal, ProposalAction, POOL_AUTHORITY_SEED, POOL_SEED, POSITION_SEED,
        PROPOSAL_SEED, STATE_VERSION,
    },
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

    /// Milestone 8: create one canonical empty position for a user in a pool.
    pub fn open_position(ctx: Context<OpenPosition>) -> Result<()> {
        let position = &mut ctx.accounts.position;
        position.version = STATE_VERSION;
        position.pool = ctx.accounts.pool.key();
        position.owner = ctx.accounts.user.key();
        position.bump = ctx.bumps.position;
        position.staked_amount = 0;
        position.reward_debt_scaled = 0;
        position.pending_reward_scaled = 0;

        emit!(PositionOpened {
            pool: position.pool,
            position: position.key(),
            owner: position.owner,
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 8: close only an empty, reward-free position owned by signer.
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        let position = &ctx.accounts.position;
        require_keys_eq!(
            position.owner,
            ctx.accounts.user.key(),
            StakingError::Unauthorized
        );
        require_keys_eq!(
            position.pool,
            ctx.accounts.pool.key(),
            StakingError::Unauthorized
        );
        require!(
            position.staked_amount == 0
                && position.reward_debt_scaled == 0
                && position.pending_reward_scaled == 0,
            StakingError::PositionNotEmpty
        );

        emit!(PositionClosed {
            pool: position.pool,
            position: position.key(),
            owner: position.owner,
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 9: checkpoint, transfer real REWARD tokens, then credit budget.
    pub fn fund_rewards(ctx: Context<FundRewards>, amount: u64) -> Result<()> {
        require_positive_amount(amount)?;
        require_original_token_program(&ctx.accounts.token_program)?;
        require_pool_vaults(&ctx.accounts.pool, &ctx.accounts.reward_vault, None)?;

        checkpoint_pool(&mut ctx.accounts.pool)?;

        token::transfer_checked(
            ctx.accounts.fund_rewards_transfer_context(),
            amount,
            TOKEN_DECIMALS,
        )?;

        let funded_scaled = checked_scale_base_units(amount)?;
        ctx.accounts.pool.remaining_reward_budget_scaled = crate::math::checked_add_scaled(
            ctx.accounts.pool.remaining_reward_budget_scaled,
            funded_scaled,
        )?;

        emit!(RewardsFunded {
            pool: ctx.accounts.pool.key(),
            funder: ctx.accounts.source_authority.key(),
            source_reward_account: ctx.accounts.source_reward_account.key(),
            amount,
            remaining_reward_budget_scaled: ctx.accounts.pool.remaining_reward_budget_scaled,
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 10: stake canonical STAKE ATA principal after checkpoint and settlement.
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require_positive_amount(amount)?;
        require!(!ctx.accounts.pool.paused, StakingError::PoolPaused);
        require_original_token_program(&ctx.accounts.token_program)?;
        require_position_owner(
            &ctx.accounts.position,
            &ctx.accounts.pool,
            &ctx.accounts.user,
        )?;
        require_pool_vaults(
            &ctx.accounts.pool,
            &ctx.accounts.reward_vault,
            Some(&ctx.accounts.stake_vault),
        )?;

        checkpoint_pool(&mut ctx.accounts.pool)?;
        settle_position(&mut ctx.accounts.position, &ctx.accounts.pool)?;

        token::transfer_checked(
            ctx.accounts.stake_transfer_context(),
            amount,
            TOKEN_DECIMALS,
        )?;

        ctx.accounts.position.staked_amount =
            checked_add_u64(ctx.accounts.position.staked_amount, amount)?;
        ctx.accounts.pool.total_staked = checked_add_u64(ctx.accounts.pool.total_staked, amount)?;
        ctx.accounts.position.reward_debt_scaled = reset_position_reward_debt(
            ctx.accounts.position.staked_amount,
            ctx.accounts.pool.acc_reward_per_stake_scaled,
        )?;

        emit!(Staked {
            pool: ctx.accounts.pool.key(),
            position: ctx.accounts.position.key(),
            owner: ctx.accounts.user.key(),
            amount,
            position_staked_amount: ctx.accounts.position.staked_amount,
            total_staked: ctx.accounts.pool.total_staked,
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 11: unstake principal only, preserving settled rewards.
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        require_positive_amount(amount)?;
        require_original_token_program(&ctx.accounts.token_program)?;
        require_position_owner(
            &ctx.accounts.position,
            &ctx.accounts.pool,
            &ctx.accounts.user,
        )?;
        require_pool_vaults(
            &ctx.accounts.pool,
            &ctx.accounts.reward_vault,
            Some(&ctx.accounts.stake_vault),
        )?;
        require!(
            ctx.accounts.position.staked_amount >= amount,
            StakingError::InsufficientStake
        );

        checkpoint_pool(&mut ctx.accounts.pool)?;
        settle_position(&mut ctx.accounts.position, &ctx.accounts.pool)?;

        ctx.accounts.position.staked_amount =
            checked_sub_u64(ctx.accounts.position.staked_amount, amount)?;
        ctx.accounts.pool.total_staked = checked_sub_u64(ctx.accounts.pool.total_staked, amount)?;
        ctx.accounts.position.reward_debt_scaled = reset_position_reward_debt(
            ctx.accounts.position.staked_amount,
            ctx.accounts.pool.acc_reward_per_stake_scaled,
        )?;

        let pool_key = ctx.accounts.pool.key();
        let pool_authority_bump = [ctx.accounts.pool.pool_authority_bump];
        let signer_seeds: &[&[&[u8]]] = &[&[
            POOL_AUTHORITY_SEED,
            pool_key.as_ref(),
            pool_authority_bump.as_ref(),
        ]];

        token::transfer_checked(
            ctx.accounts
                .unstake_transfer_context()
                .with_signer(signer_seeds),
            amount,
            TOKEN_DECIMALS,
        )?;

        emit!(Unstaked {
            pool: ctx.accounts.pool.key(),
            position: ctx.accounts.position.key(),
            owner: ctx.accounts.user.key(),
            amount,
            position_staked_amount: ctx.accounts.position.staked_amount,
            total_staked: ctx.accounts.pool.total_staked,
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 12: claim whole REWARD base units, preserving scaled remainder.
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        require!(!ctx.accounts.pool.paused, StakingError::PoolPaused);
        require_original_token_program(&ctx.accounts.token_program)?;
        require_position_owner(
            &ctx.accounts.position,
            &ctx.accounts.pool,
            &ctx.accounts.user,
        )?;
        require_pool_vaults(&ctx.accounts.pool, &ctx.accounts.reward_vault, None)?;

        checkpoint_pool(&mut ctx.accounts.pool)?;
        settle_position(&mut ctx.accounts.position, &ctx.accounts.pool)?;

        let claimable_base_units =
            checked_claimable_base_units(ctx.accounts.position.pending_reward_scaled)?;
        if claimable_base_units == 0 {
            return err!(StakingError::NothingToClaim);
        }
        require!(
            ctx.accounts.reward_vault.amount >= claimable_base_units,
            StakingError::InsufficientRewardBacking
        );

        let claim_output = claim_position_rewards(ClaimPositionInput {
            pending_reward_scaled: ctx.accounts.position.pending_reward_scaled,
            allocated_liability_scaled: ctx.accounts.pool.allocated_liability_scaled,
        })?;
        let paid_scaled = checked_paid_scaled(claim_output.claimed_base_units)?;

        ctx.accounts.position.pending_reward_scaled = claim_output.pending_reward_scaled;
        ctx.accounts.pool.allocated_liability_scaled =
            checked_sub_scaled(ctx.accounts.pool.allocated_liability_scaled, paid_scaled)?;

        let pool_key = ctx.accounts.pool.key();
        let pool_authority_bump = [ctx.accounts.pool.pool_authority_bump];
        let signer_seeds: &[&[&[u8]]] = &[&[
            POOL_AUTHORITY_SEED,
            pool_key.as_ref(),
            pool_authority_bump.as_ref(),
        ]];

        token::transfer_checked(
            ctx.accounts
                .claim_transfer_context()
                .with_signer(signer_seeds),
            claim_output.claimed_base_units,
            TOKEN_DECIMALS,
        )?;

        emit!(RewardsClaimed {
            pool: ctx.accounts.pool.key(),
            position: ctx.accounts.position.key(),
            owner: ctx.accounts.user.key(),
            amount: claim_output.claimed_base_units,
            paid_scaled,
            pending_reward_scaled: ctx.accounts.position.pending_reward_scaled,
            allocated_liability_scaled: ctx.accounts.pool.allocated_liability_scaled,
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 13: any current admin can checkpoint and immediately pause.
    pub fn pause_pool(ctx: Context<PausePool>) -> Result<()> {
        require_current_admin(&ctx.accounts.pool, &ctx.accounts.admin.key())?;
        require!(!ctx.accounts.pool.paused, StakingError::PoolNotPaused);

        checkpoint_pool(&mut ctx.accounts.pool)?;
        ctx.accounts.pool.paused = true;

        emit!(PoolPaused {
            pool: ctx.accounts.pool.key(),
            admin: ctx.accounts.admin.key(),
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 14: return all principal and recycle the user's reward entitlement.
    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        require_original_token_program(&ctx.accounts.token_program)?;
        require_position_owner(
            &ctx.accounts.position,
            &ctx.accounts.pool,
            &ctx.accounts.user,
        )?;
        require_pool_vaults(
            &ctx.accounts.pool,
            &ctx.accounts.reward_vault,
            Some(&ctx.accounts.stake_vault),
        )?;

        checkpoint_pool(&mut ctx.accounts.pool)?;
        settle_position(&mut ctx.accounts.position, &ctx.accounts.pool)?;
        require!(
            ctx.accounts.position.staked_amount > 0
                || ctx.accounts.position.pending_reward_scaled > 0,
            StakingError::InvalidAmount
        );

        let withdrawn_amount = ctx.accounts.position.staked_amount;
        let forfeited_before = ctx.accounts.position.pending_reward_scaled;
        let forfeit = forfeit_position_rewards(ForfeitPositionInput {
            pending_reward_scaled: ctx.accounts.position.pending_reward_scaled,
            remaining_reward_budget_scaled: ctx.accounts.pool.remaining_reward_budget_scaled,
            allocated_liability_scaled: ctx.accounts.pool.allocated_liability_scaled,
        })?;

        ctx.accounts.position.staked_amount = 0;
        ctx.accounts.position.pending_reward_scaled = forfeit.pending_reward_scaled;
        ctx.accounts.position.reward_debt_scaled = 0;
        ctx.accounts.pool.remaining_reward_budget_scaled = forfeit.remaining_reward_budget_scaled;
        ctx.accounts.pool.allocated_liability_scaled = forfeit.allocated_liability_scaled;
        ctx.accounts.pool.total_staked =
            checked_sub_u64(ctx.accounts.pool.total_staked, withdrawn_amount)?;

        if withdrawn_amount > 0 {
            let pool_key = ctx.accounts.pool.key();
            let pool_authority_bump = [ctx.accounts.pool.pool_authority_bump];
            let signer_seeds: &[&[&[u8]]] = &[&[
                POOL_AUTHORITY_SEED,
                pool_key.as_ref(),
                pool_authority_bump.as_ref(),
            ]];

            token::transfer_checked(
                ctx.accounts
                    .emergency_withdraw_transfer_context()
                    .with_signer(signer_seeds),
                withdrawn_amount,
                TOKEN_DECIMALS,
            )?;
        }

        emit!(EmergencyWithdrawn {
            pool: ctx.accounts.pool.key(),
            position: ctx.accounts.position.key(),
            owner: ctx.accounts.user.key(),
            amount: withdrawn_amount,
            forfeited_scaled: forfeited_before,
            total_staked: ctx.accounts.pool.total_staked,
            remaining_reward_budget_scaled: ctx.accounts.pool.remaining_reward_budget_scaled,
            allocated_liability_scaled: ctx.accounts.pool.allocated_liability_scaled,
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 15: create one immutable allowlisted admin proposal.
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_id: u64,
        action: ProposalAction,
    ) -> Result<()> {
        let creator_index = require_current_admin(&ctx.accounts.pool, &ctx.accounts.creator.key())?;
        require!(
            proposal_id == ctx.accounts.pool.next_proposal_id,
            StakingError::InvalidProposalId
        );
        validate_proposal_action(&ctx.accounts.pool, &action)?;

        let current_slot = Clock::get()?.slot;
        let mut approvals = [false; ADMIN_COUNT];
        approvals[creator_index] = true;

        let proposal = &mut ctx.accounts.proposal;
        proposal.version = STATE_VERSION;
        proposal.pool = ctx.accounts.pool.key();
        proposal.proposal_id = proposal_id;
        proposal.creator = ctx.accounts.creator.key();
        proposal.admin_epoch = ctx.accounts.pool.admin_epoch;
        proposal.action = action;
        proposal.approvals = approvals;
        proposal.approval_count = 1;
        proposal.created_slot = current_slot;
        proposal.expires_at_slot = checked_add_u64(current_slot, PROPOSAL_TTL_SLOTS)?;
        proposal.executed = false;
        proposal.bump = ctx.bumps.proposal;

        ctx.accounts.pool.next_proposal_id =
            checked_add_u64(ctx.accounts.pool.next_proposal_id, 1)?;

        emit!(ProposalCreated {
            pool: proposal.pool,
            proposal: proposal.key(),
            proposal_id,
            creator: proposal.creator,
            admin_epoch: proposal.admin_epoch,
            expires_at_slot: proposal.expires_at_slot,
            slot: current_slot,
        });

        Ok(())
    }

    /// Milestone 15: add one distinct current-admin approval to a proposal.
    pub fn approve_proposal(ctx: Context<ApproveProposal>) -> Result<()> {
        let admin_index = require_current_admin(&ctx.accounts.pool, &ctx.accounts.admin.key())?;
        require_proposal_live(&ctx.accounts.pool, &ctx.accounts.proposal)?;
        require!(
            !ctx.accounts.proposal.approvals[admin_index],
            StakingError::DuplicateApproval
        );

        ctx.accounts.proposal.approvals[admin_index] = true;
        ctx.accounts.proposal.approval_count = ctx
            .accounts
            .proposal
            .approval_count
            .checked_add(1)
            .ok_or(StakingError::ArithmeticOverflow)?;

        emit!(ProposalApproved {
            pool: ctx.accounts.pool.key(),
            proposal: ctx.accounts.proposal.key(),
            proposal_id: ctx.accounts.proposal.proposal_id,
            admin: ctx.accounts.admin.key(),
            approval_count: ctx.accounts.proposal.approval_count,
            admin_epoch: ctx.accounts.proposal.admin_epoch,
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 16: execute exactly one approved proposal action once.
    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        require_proposal_live(&ctx.accounts.pool, &ctx.accounts.proposal)?;
        require!(
            ctx.accounts.proposal.approval_count >= ADMIN_THRESHOLD,
            StakingError::ProposalNotApproved
        );

        let action = ctx.accounts.proposal.action.clone();
        match action {
            ProposalAction::SetRewardRate { new_rate } => {
                require!(
                    new_rate <= ctx.accounts.pool.max_reward_rate_per_slot,
                    StakingError::RewardRateAboveMaximum
                );
                checkpoint_pool(&mut ctx.accounts.pool)?;
                ctx.accounts.pool.reward_rate_per_slot = new_rate;
                emit!(RewardRateChanged {
                    pool: ctx.accounts.pool.key(),
                    proposal: ctx.accounts.proposal.key(),
                    proposal_id: ctx.accounts.proposal.proposal_id,
                    new_rate,
                    slot: Clock::get()?.slot,
                });
            }
            ProposalAction::UnpausePool => {
                require!(ctx.accounts.pool.paused, StakingError::PoolNotPaused);
                ctx.accounts.pool.last_update_slot = Clock::get()?.slot;
                ctx.accounts.pool.paused = false;
                emit!(PoolUnpaused {
                    pool: ctx.accounts.pool.key(),
                    proposal: ctx.accounts.proposal.key(),
                    proposal_id: ctx.accounts.proposal.proposal_id,
                    slot: ctx.accounts.pool.last_update_slot,
                });
            }
            ProposalAction::ReplaceAdmin {
                old_admin,
                new_admin,
            } => {
                let admin_index =
                    validate_admin_replacement(&ctx.accounts.pool, &old_admin, &new_admin)?;
                ctx.accounts.pool.admins[admin_index] = new_admin;
                ctx.accounts.pool.admin_epoch = checked_add_u64(ctx.accounts.pool.admin_epoch, 1)?;
                emit!(AdminReplaced {
                    pool: ctx.accounts.pool.key(),
                    proposal: ctx.accounts.proposal.key(),
                    proposal_id: ctx.accounts.proposal.proposal_id,
                    old_admin,
                    new_admin,
                    admin_epoch: ctx.accounts.pool.admin_epoch,
                    slot: Clock::get()?.slot,
                });
            }
        }

        ctx.accounts.proposal.executed = true;
        emit!(ProposalExecuted {
            pool: ctx.accounts.pool.key(),
            proposal: ctx.accounts.proposal.key(),
            proposal_id: ctx.accounts.proposal.proposal_id,
            admin_epoch: ctx.accounts.proposal.admin_epoch,
            slot: Clock::get()?.slot,
        });

        Ok(())
    }

    /// Milestone 16: close only executed, expired, or stale proposals to creator.
    pub fn close_proposal(ctx: Context<CloseProposal>) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let stale = ctx.accounts.proposal.admin_epoch != ctx.accounts.pool.admin_epoch;
        let expired = current_slot > ctx.accounts.proposal.expires_at_slot;
        require!(
            ctx.accounts.proposal.executed || stale || expired,
            StakingError::ProposalNotApproved
        );

        emit!(ProposalClosed {
            pool: ctx.accounts.pool.key(),
            proposal: ctx.accounts.proposal.key(),
            proposal_id: ctx.accounts.proposal.proposal_id,
            creator: ctx.accounts.proposal.creator,
            slot: current_slot,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    // create pool account
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
    // create stake vault token account
    #[account(
        init,
        payer = initializer,
        associated_token::mint = stake_mint,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program
    )]
    pub stake_vault: Account<'info, TokenAccount>,
    // create reward vault token account
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

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub pool: Account<'info, Pool>,
    #[account(
        init,
        payer = user,
        space = Position::SPACE,
        seeds = [POSITION_SEED, pool.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        close = user,
        seeds = [POSITION_SEED, pool.key().as_ref(), user.key().as_ref()],
        bump = position.bump,
    )]
    pub position: Account<'info, Position>,
}

#[derive(Accounts)]
pub struct FundRewards<'info> {
    #[account(mut)]
    pub source_authority: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut, token::mint = reward_mint, token::authority = source_authority)]
    pub source_reward_account: Account<'info, TokenAccount>,
    #[account(address = pool.reward_mint)]
    pub reward_mint: Account<'info, Mint>,
    #[account(mut, address = pool.reward_vault, token::mint = reward_mint)]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [POSITION_SEED, pool.key().as_ref(), user.key().as_ref()],
        bump = position.bump,
    )]
    pub position: Account<'info, Position>,
    #[account(address = pool.stake_mint)]
    pub stake_mint: Account<'info, Mint>,
    #[account(address = pool.reward_mint)]
    pub reward_mint: Account<'info, Mint>,
    #[account(mut, associated_token::mint = stake_mint, associated_token::authority = user)]
    pub user_stake_account: Account<'info, TokenAccount>,
    #[account(mut, address = pool.stake_vault, token::mint = stake_mint)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(mut, address = pool.reward_vault, token::mint = reward_mint)]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    /// CHECK: Milestone 11: canonical PDA validated by seeds and used only as token signer.
    #[account(seeds = [POOL_AUTHORITY_SEED, pool.key().as_ref()], bump = pool.pool_authority_bump)]
    pub pool_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [POSITION_SEED, pool.key().as_ref(), user.key().as_ref()],
        bump = position.bump,
    )]
    pub position: Account<'info, Position>,
    #[account(address = pool.stake_mint)]
    pub stake_mint: Account<'info, Mint>,
    #[account(address = pool.reward_mint)]
    pub reward_mint: Account<'info, Mint>,
    #[account(mut, associated_token::mint = stake_mint, associated_token::authority = user)]
    pub user_stake_account: Account<'info, TokenAccount>,
    #[account(mut, address = pool.stake_vault, token::mint = stake_mint, token::authority = pool_authority)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(mut, address = pool.reward_vault, token::mint = reward_mint)]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    /// CHECK: Milestone 12: canonical PDA validated by seeds and used only as token signer.
    #[account(seeds = [POOL_AUTHORITY_SEED, pool.key().as_ref()], bump = pool.pool_authority_bump)]
    pub pool_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [POSITION_SEED, pool.key().as_ref(), user.key().as_ref()],
        bump = position.bump,
    )]
    pub position: Account<'info, Position>,
    #[account(address = pool.reward_mint)]
    pub reward_mint: Account<'info, Mint>,
    #[account(mut, address = pool.reward_vault, token::mint = reward_mint, token::authority = pool_authority)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = reward_mint, associated_token::authority = user)]
    pub user_reward_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PausePool<'info> {
    pub admin: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    /// CHECK: Milestone 14: canonical PDA validated by seeds and used only as token signer.
    #[account(seeds = [POOL_AUTHORITY_SEED, pool.key().as_ref()], bump = pool.pool_authority_bump)]
    pub pool_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [POSITION_SEED, pool.key().as_ref(), user.key().as_ref()],
        bump = position.bump,
    )]
    pub position: Account<'info, Position>,
    #[account(address = pool.stake_mint)]
    pub stake_mint: Account<'info, Mint>,
    #[account(address = pool.reward_mint)]
    pub reward_mint: Account<'info, Mint>,
    #[account(mut, associated_token::mint = stake_mint, associated_token::authority = user)]
    pub user_stake_account: Account<'info, TokenAccount>,
    #[account(mut, address = pool.stake_vault, token::mint = stake_mint, token::authority = pool_authority)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(mut, address = pool.reward_vault, token::mint = reward_mint)]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(
        init,
        payer = creator,
        space = Proposal::SPACE,
        seeds = [PROPOSAL_SEED, pool.key().as_ref(), &proposal_id.to_le_bytes()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveProposal<'info> {
    pub admin: Signer<'info>,
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [PROPOSAL_SEED, pool.key().as_ref(), &proposal.proposal_id.to_le_bytes()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [PROPOSAL_SEED, pool.key().as_ref(), &proposal.proposal_id.to_le_bytes()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct CloseProposal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        close = creator,
        seeds = [PROPOSAL_SEED, pool.key().as_ref(), &proposal.proposal_id.to_le_bytes()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,
    /// CHECK: Milestone 16: receives rent and must match the proposal creator.
    #[account(mut, address = proposal.creator)]
    pub creator: UncheckedAccount<'info>,
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

#[event]
pub struct PositionOpened {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub slot: u64,
}

#[event]
pub struct PositionClosed {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub slot: u64,
}

#[event]
pub struct RewardsFunded {
    pub pool: Pubkey,
    pub funder: Pubkey,
    pub source_reward_account: Pubkey,
    pub amount: u64,
    pub remaining_reward_budget_scaled: u128,
    pub slot: u64,
}

#[event]
pub struct Staked {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub position_staked_amount: u64,
    pub total_staked: u64,
    pub slot: u64,
}

#[event]
pub struct Unstaked {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub position_staked_amount: u64,
    pub total_staked: u64,
    pub slot: u64,
}

#[event]
pub struct RewardsClaimed {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub paid_scaled: u128,
    pub pending_reward_scaled: u128,
    pub allocated_liability_scaled: u128,
    pub slot: u64,
}

#[event]
pub struct PoolPaused {
    pub pool: Pubkey,
    pub admin: Pubkey,
    pub slot: u64,
}

#[event]
pub struct EmergencyWithdrawn {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub forfeited_scaled: u128,
    pub total_staked: u64,
    pub remaining_reward_budget_scaled: u128,
    pub allocated_liability_scaled: u128,
    pub slot: u64,
}

#[event]
pub struct ProposalCreated {
    pub pool: Pubkey,
    pub proposal: Pubkey,
    pub proposal_id: u64,
    pub creator: Pubkey,
    pub admin_epoch: u64,
    pub expires_at_slot: u64,
    pub slot: u64,
}

#[event]
pub struct ProposalApproved {
    pub pool: Pubkey,
    pub proposal: Pubkey,
    pub proposal_id: u64,
    pub admin: Pubkey,
    pub approval_count: u8,
    pub admin_epoch: u64,
    pub slot: u64,
}

#[event]
pub struct ProposalExecuted {
    pub pool: Pubkey,
    pub proposal: Pubkey,
    pub proposal_id: u64,
    pub admin_epoch: u64,
    pub slot: u64,
}

#[event]
pub struct ProposalClosed {
    pub pool: Pubkey,
    pub proposal: Pubkey,
    pub proposal_id: u64,
    pub creator: Pubkey,
    pub slot: u64,
}

#[event]
pub struct RewardRateChanged {
    pub pool: Pubkey,
    pub proposal: Pubkey,
    pub proposal_id: u64,
    pub new_rate: u64,
    pub slot: u64,
}

#[event]
pub struct PoolUnpaused {
    pub pool: Pubkey,
    pub proposal: Pubkey,
    pub proposal_id: u64,
    pub slot: u64,
}

#[event]
pub struct AdminReplaced {
    pub pool: Pubkey,
    pub proposal: Pubkey,
    pub proposal_id: u64,
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
    pub admin_epoch: u64,
    pub slot: u64,
}

impl<'info> FundRewards<'info> {
    fn fund_rewards_transfer_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.source_reward_account.to_account_info(),
                mint: self.reward_mint.to_account_info(),
                to: self.reward_vault.to_account_info(),
                authority: self.source_authority.to_account_info(),
            },
        )
    }
}

impl<'info> Stake<'info> {
    fn stake_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.user_stake_account.to_account_info(),
                mint: self.stake_mint.to_account_info(),
                to: self.stake_vault.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }
}

impl<'info> Unstake<'info> {
    fn unstake_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.stake_vault.to_account_info(),
                mint: self.stake_mint.to_account_info(),
                to: self.user_stake_account.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            },
        )
    }
}

impl<'info> ClaimRewards<'info> {
    fn claim_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.reward_vault.to_account_info(),
                mint: self.reward_mint.to_account_info(),
                to: self.user_reward_account.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            },
        )
    }
}

impl<'info> EmergencyWithdraw<'info> {
    fn emergency_withdraw_transfer_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.stake_vault.to_account_info(),
                mint: self.stake_mint.to_account_info(),
                to: self.user_stake_account.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            },
        )
    }
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

fn require_original_token_program(token_program: &Program<Token>) -> Result<()> {
    require_keys_eq!(
        token_program.key(),
        anchor_spl::token::ID,
        StakingError::InvalidTokenProgram
    );
    Ok(())
}

fn require_current_admin(pool: &Account<Pool>, admin: &Pubkey) -> Result<usize> {
    let Some(index) = pool.admins.iter().position(|stored| stored == admin) else {
        return err!(StakingError::Unauthorized);
    };

    Ok(index)
}

fn validate_proposal_action(pool: &Account<Pool>, action: &ProposalAction) -> Result<()> {
    match action {
        ProposalAction::SetRewardRate { new_rate } => {
            require!(
                *new_rate <= pool.max_reward_rate_per_slot,
                StakingError::RewardRateAboveMaximum
            );
        }
        ProposalAction::UnpausePool => {}
        ProposalAction::ReplaceAdmin {
            old_admin,
            new_admin,
        } => {
            validate_admin_replacement(pool, old_admin, new_admin)?;
        }
    }

    Ok(())
}

fn validate_admin_replacement(
    pool: &Account<Pool>,
    old_admin: &Pubkey,
    new_admin: &Pubkey,
) -> Result<usize> {
    require_keys_neq!(
        *new_admin,
        Pubkey::default(),
        StakingError::InvalidAdminReplacement
    );
    let Some(old_index) = pool.admins.iter().position(|admin| admin == old_admin) else {
        return err!(StakingError::InvalidAdminReplacement);
    };
    require!(
        pool.admins.iter().all(|admin| admin != new_admin),
        StakingError::InvalidAdminReplacement
    );

    Ok(old_index)
}

fn require_proposal_live(pool: &Account<Pool>, proposal: &Account<Proposal>) -> Result<()> {
    require_keys_eq!(proposal.pool, pool.key(), StakingError::Unauthorized);
    require!(!proposal.executed, StakingError::ProposalAlreadyExecuted);
    require!(
        proposal.admin_epoch == pool.admin_epoch,
        StakingError::StaleProposal
    );
    require!(
        Clock::get()?.slot <= proposal.expires_at_slot,
        StakingError::ProposalExpired
    );
    Ok(())
}

fn require_position_owner(
    position: &Account<Position>,
    pool: &Account<Pool>,
    user: &Signer,
) -> Result<()> {
    require_keys_eq!(position.pool, pool.key(), StakingError::Unauthorized);
    require_keys_eq!(position.owner, user.key(), StakingError::Unauthorized);
    Ok(())
}

fn require_pool_vaults(
    pool: &Account<Pool>,
    reward_vault: &Account<TokenAccount>,
    stake_vault: Option<&Account<TokenAccount>>,
) -> Result<()> {
    let pool_key = pool.key();
    let pool_authority = Pubkey::create_program_address(
        &[
            POOL_AUTHORITY_SEED,
            pool_key.as_ref(),
            &[pool.pool_authority_bump],
        ],
        &crate::ID,
    )
    .map_err(|_| StakingError::Unauthorized)?;

    require_keys_eq!(
        reward_vault.key(),
        pool.reward_vault,
        StakingError::Unauthorized
    );
    require_keys_eq!(
        reward_vault.mint,
        pool.reward_mint,
        StakingError::Unauthorized
    );
    require_keys_eq!(
        reward_vault.owner,
        pool_authority,
        StakingError::Unauthorized
    );

    if let Some(stake_vault) = stake_vault {
        require_keys_eq!(
            stake_vault.key(),
            pool.stake_vault,
            StakingError::Unauthorized
        );
        require_keys_eq!(
            stake_vault.mint,
            pool.stake_mint,
            StakingError::Unauthorized
        );
        require_keys_eq!(
            stake_vault.owner,
            pool_authority,
            StakingError::Unauthorized
        );
    }

    Ok(())
}

fn checkpoint_pool(pool: &mut Account<Pool>) -> Result<()> {
    let checkpoint = checkpoint_pool_rewards(PoolCheckpointInput {
        current_slot: Clock::get()?.slot,
        last_update_slot: pool.last_update_slot,
        reward_rate_per_slot_base_units: pool.reward_rate_per_slot,
        total_staked_base_units: pool.total_staked,
        acc_reward_per_stake_scaled: pool.acc_reward_per_stake_scaled,
        remaining_reward_budget_scaled: pool.remaining_reward_budget_scaled,
        allocated_liability_scaled: pool.allocated_liability_scaled,
        paused: pool.paused,
    })?;

    pool.acc_reward_per_stake_scaled = checkpoint.acc_reward_per_stake_scaled;
    pool.remaining_reward_budget_scaled = checkpoint.remaining_reward_budget_scaled;
    pool.allocated_liability_scaled = checkpoint.allocated_liability_scaled;
    pool.last_update_slot = checkpoint.last_update_slot;

    Ok(())
}

fn settle_position(position: &mut Account<Position>, pool: &Account<Pool>) -> Result<()> {
    let settlement = settle_position_rewards(PositionSettlementInput {
        staked_amount_base_units: position.staked_amount,
        reward_debt_scaled: position.reward_debt_scaled,
        pending_reward_scaled: position.pending_reward_scaled,
        acc_reward_per_stake_scaled: pool.acc_reward_per_stake_scaled,
    })?;

    position.pending_reward_scaled = settlement.pending_reward_scaled;
    position.reward_debt_scaled = settlement.reward_debt_scaled;

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
