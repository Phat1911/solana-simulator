#![forbid(unsafe_code)]

//! Milestone 1: behavior-free scaffold for the future Devnet-only faucet.
//!
//! The one-claim receipt account and SPL mint CPI are deliberately deferred to
//! the faucet implementation milestone.

pub const PROGRAM_NAME: &str = "demo_faucet";
pub const PROGRAM_VERSION: u8 = 1;

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
}
