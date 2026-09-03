#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

//! Milestone 17: Devnet-only faucet for one fixed STAKE allocation per wallet.

use anchor_lang::{prelude::*, solana_program::program_option::COption};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};

declare_id!("J12YAqC7dWbVWVAFveRdJ8SJ3sYc6roP3WJekhy4bDkM");

pub const PROGRAM_NAME: &str = "demo_faucet";
pub const PROGRAM_VERSION: u8 = 1;
pub const STATE_VERSION: u8 = 1;
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

pub const TOKEN_DECIMALS: u8 = 6;
pub const BASE_UNITS_PER_TOKEN: u64 = 1_000_000;
pub const FAUCET_CLAIM_AMOUNT: u64 = 1_000 * BASE_UNITS_PER_TOKEN;

pub const FAUCET_AUTHORITY_SEED: &[u8] = b"faucet-authority";
pub const FAUCET_CLAIM_SEED: &[u8] = b"faucet-claim";

#[program]
pub mod demo_faucet {
    use super::*;

    /// Milestone 17: mint exactly 1,000 STAKE once to the claimant's canonical ATA.
    pub fn claim_test_stake(ctx: Context<ClaimTestStake>) -> Result<()> {
        require!(
            ctx.accounts.stake_mint.decimals == TOKEN_DECIMALS,
            FaucetError::InvalidTokenDecimals
        );
        require!(
            ctx.accounts.stake_mint.mint_authority
                == COption::Some(ctx.accounts.faucet_authority.key()),
            FaucetError::InvalidMintAuthority
        );
        require!(
            ctx.accounts.stake_mint.freeze_authority == COption::None,
            FaucetError::InvalidMintAuthority
        );
        require_original_token_program(&ctx.accounts.token_program)?;

        let stake_mint_key = ctx.accounts.stake_mint.key();
        let faucet_authority_bump = [ctx.bumps.faucet_authority];
        let signer_seeds: &[&[&[u8]]] = &[&[
            FAUCET_AUTHORITY_SEED,
            stake_mint_key.as_ref(),
            faucet_authority_bump.as_ref(),
        ]];

        token::mint_to(
            ctx.accounts.mint_to_context().with_signer(signer_seeds),
            FAUCET_CLAIM_AMOUNT,
        )?;

        let receipt = &mut ctx.accounts.claim_receipt;
        receipt.version = STATE_VERSION;
        receipt.stake_mint = stake_mint_key;
        receipt.claimant = ctx.accounts.claimant.key();
        receipt.amount = FAUCET_CLAIM_AMOUNT;
        receipt.claimed_slot = Clock::get()?.slot;
        receipt.bump = ctx.bumps.claim_receipt;

        emit!(FaucetClaimed {
            stake_mint: receipt.stake_mint,
            claimant: receipt.claimant,
            claim_receipt: receipt.key(),
            destination: ctx.accounts.claimant_stake_account.key(),
            amount: receipt.amount,
            slot: receipt.claimed_slot,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ClaimTestStake<'info> {
    #[account(mut)]
    pub claimant: Signer<'info>,
    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,
    /// CHECK: Milestone 17: this PDA has no data account; it only signs mint CPIs.
    #[account(seeds = [FAUCET_AUTHORITY_SEED, stake_mint.key().as_ref()], bump)]
    pub faucet_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = claimant,
        associated_token::mint = stake_mint,
        associated_token::authority = claimant,
        associated_token::token_program = token_program
    )]
    pub claimant_stake_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = claimant,
        space = FaucetClaimReceipt::SPACE,
        seeds = [FAUCET_CLAIM_SEED, stake_mint.key().as_ref(), claimant.key().as_ref()],
        bump
    )]
    pub claim_receipt: Account<'info, FaucetClaimReceipt>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug, PartialEq, Eq, InitSpace)]
pub struct FaucetClaimReceipt {
    pub version: u8,
    pub stake_mint: Pubkey,
    pub claimant: Pubkey,
    pub amount: u64,
    pub claimed_slot: u64,
    pub bump: u8,
}

impl FaucetClaimReceipt {
    pub const LEN: usize = <Self as Space>::INIT_SPACE;
    pub const SPACE: usize = ANCHOR_DISCRIMINATOR_SIZE + Self::LEN;
}

#[event]
pub struct FaucetClaimed {
    pub stake_mint: Pubkey,
    pub claimant: Pubkey,
    pub claim_receipt: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
    pub slot: u64,
}

#[error_code]
#[derive(PartialEq, Eq)]
pub enum FaucetError {
    #[msg("Token mint must use exactly six decimals")]
    InvalidTokenDecimals,
    #[msg("Stake mint authority must be the Faucet Authority PDA")]
    InvalidMintAuthority,
    #[msg("Only the original SPL Token Program is supported")]
    InvalidTokenProgram,
    #[msg("Wallet has already claimed test stake for this mint")]
    FaucetAlreadyClaimed,
}

impl<'info> ClaimTestStake<'info> {
    fn mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.stake_mint.to_account_info(),
                to: self.claimant_stake_account.to_account_info(),
                authority: self.faucet_authority.to_account_info(),
            },
        )
    }
}

fn require_original_token_program(token_program: &Program<Token>) -> Result<()> {
    require_keys_eq!(
        token_program.key(),
        anchor_spl::token::ID,
        FaucetError::InvalidTokenProgram
    );
    Ok(())
}

pub fn derive_faucet_authority_pda(program_id: &Pubkey, stake_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[FAUCET_AUTHORITY_SEED, stake_mint.as_ref()], program_id)
}

pub fn derive_faucet_claim_pda(
    program_id: &Pubkey,
    stake_mint: &Pubkey,
    claimant: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[FAUCET_CLAIM_SEED, stake_mint.as_ref(), claimant.as_ref()],
        program_id,
    )
}

/// Milestone 2: harmless baseline check used before protocol behavior exists.
pub fn program_identity() -> (&'static str, u8) {
    (PROGRAM_NAME, PROGRAM_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faucet_program_identity_is_stable() {
        assert_eq!(program_identity(), ("demo_faucet", 1));
    }

    #[test]
    fn claim_receipt_size_is_fixed() {
        assert_eq!(FaucetClaimReceipt::LEN, 82);
        assert_eq!(FaucetClaimReceipt::SPACE, ANCHOR_DISCRIMINATOR_SIZE + 82);
    }
}
