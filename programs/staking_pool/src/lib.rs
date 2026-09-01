#![forbid(unsafe_code)]

//! Milestone 1: behavior-free scaffold for the future Anchor staking program.
//!
//! Protocol state, instruction handlers, SPL Token CPIs, and reward math are
//! intentionally introduced in later milestones after their invariants are
//! reviewed.

pub const PROGRAM_NAME: &str = "staking_pool";
pub const PROGRAM_VERSION: u8 = 1;

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
