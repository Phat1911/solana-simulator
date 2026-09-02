#![forbid(unsafe_code)]

//! Milestone 2: placeholder crate for LiteSVM integration tests.
//!
//! The first real LiteSVM tests arrive when there are account schemas and
//! instructions to execute. This crate gives the workspace a stable test target
//! today without introducing protocol behavior.

#[cfg(test)]
mod milestone7_initialize_pool;

#[cfg(test)]
mod tests {
    #[test]
    fn harness_can_link_program_crates() {
        assert_eq!(staking_pool::program_identity().0, "staking_pool");
        assert_eq!(demo_faucet::program_identity().0, "demo_faucet");
    }
}
