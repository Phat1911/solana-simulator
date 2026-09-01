//! Milestone 3: unit constants shared by pure accounting helpers.

pub const TOKEN_DECIMALS: u8 = 6;
pub const BASE_UNITS_PER_TOKEN: u64 = 1_000_000;
pub const REWARD_PRECISION: u128 = 1_000_000_000;
pub const DEVNET_MAX_REWARD_RATE_PER_SLOT: u64 = 100 * BASE_UNITS_PER_TOKEN;

pub const PROPOSAL_TTL_SLOTS: u64 = 216_000;
pub const ADMIN_COUNT: usize = 3;
pub const ADMIN_THRESHOLD: u8 = 2;
