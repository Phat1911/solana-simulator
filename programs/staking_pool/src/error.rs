//! Milestone 3: explicit staking errors shared by pure math and Anchor handlers.

use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq, Eq)]
pub enum StakingError {
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Token mint must use exactly six decimals")]
    InvalidTokenDecimals,
    #[msg("Admin set must contain exactly three distinct non-default keys")]
    InvalidAdminSet,
    #[msg("Stake and reward mints must be distinct")]
    InvalidMintPair,
    #[msg("Only the original SPL Token Program is supported")]
    InvalidTokenProgram,
    #[msg("Reward rate is above the configured Devnet maximum")]
    RewardRateAboveMaximum,
    #[msg("No whole reward base units are claimable")]
    NothingToClaim,
    #[msg("Signer is not authorized for this account")]
    Unauthorized,
    #[msg("Position still has stake or reward accounting")]
    PositionNotEmpty,
    #[msg("Pool is paused")]
    PoolPaused,
    #[msg("Pool is not paused")]
    PoolNotPaused,
    #[msg("Position does not have enough staked principal")]
    InsufficientStake,
    #[msg("Reward vault cannot back the requested payout")]
    InsufficientRewardBacking,
}
