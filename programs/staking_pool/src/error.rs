//! Milestone 3: explicit pure-math errors used before Anchor errors exist.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StakingError {
    ArithmeticOverflow,
    ArithmeticUnderflow,
    InvalidAmount,
    NothingToClaim,
}
