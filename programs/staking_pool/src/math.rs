//! Milestone 3: pure checked arithmetic for token and scaled reward units.

use crate::constants::REWARD_PRECISION;
use crate::error::StakingError;

pub type StakingResult<T> = Result<T, StakingError>;

pub fn require_positive_amount(amount: u64) -> StakingResult<u64> {
    if amount == 0 {
        Err(StakingError::InvalidAmount)
    } else {
        Ok(amount)
    }
}

pub fn checked_add_u64(left: u64, right: u64) -> StakingResult<u64> {
    left.checked_add(right)
        .ok_or(StakingError::ArithmeticOverflow)
}

pub fn checked_sub_u64(left: u64, right: u64) -> StakingResult<u64> {
    left.checked_sub(right)
        .ok_or(StakingError::ArithmeticUnderflow)
}

pub fn checked_add_scaled(left: u128, right: u128) -> StakingResult<u128> {
    left.checked_add(right)
        .ok_or(StakingError::ArithmeticOverflow)
}

pub fn checked_sub_scaled(left: u128, right: u128) -> StakingResult<u128> {
    left.checked_sub(right)
        .ok_or(StakingError::ArithmeticUnderflow)
}

pub fn checked_mul_u64_to_u128(left: u64, right: u64) -> StakingResult<u128> {
    u128::from(left)
        .checked_mul(u128::from(right))
        .ok_or(StakingError::ArithmeticOverflow)
}

pub fn checked_scale_base_units(base_units: u64) -> StakingResult<u128> {
    u128::from(base_units)
        .checked_mul(REWARD_PRECISION)
        .ok_or(StakingError::ArithmeticOverflow)
}

pub fn checked_emission_scaled(
    elapsed_slots: u64,
    reward_rate_per_slot_base_units: u64,
) -> StakingResult<u128> {
    checked_mul_u64_to_u128(elapsed_slots, reward_rate_per_slot_base_units)?
        .checked_mul(REWARD_PRECISION)
        .ok_or(StakingError::ArithmeticOverflow)
}

pub fn checked_claimable_base_units(pending_reward_scaled: u128) -> StakingResult<u64> {
    checked_u128_to_u64(pending_reward_scaled / REWARD_PRECISION)
}

pub fn checked_paid_scaled(claimable_base_units: u64) -> StakingResult<u128> {
    checked_scale_base_units(claimable_base_units)
}

pub fn checked_u128_to_u64(value: u128) -> StakingResult<u64> {
    u64::try_from(value).map_err(|_| StakingError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        BASE_UNITS_PER_TOKEN, DEVNET_MAX_REWARD_RATE_PER_SLOT, REWARD_PRECISION, TOKEN_DECIMALS,
    };

    #[test]
    fn constants_match_six_decimal_reward_model() {
        assert_eq!(TOKEN_DECIMALS, 6);
        assert_eq!(BASE_UNITS_PER_TOKEN, 1_000_000);
        assert_eq!(REWARD_PRECISION, 1_000_000_000);
        assert_eq!(DEVNET_MAX_REWARD_RATE_PER_SLOT, 100_000_000);
    }

    #[test]
    fn zero_amount_is_invalid_for_user_supplied_amounts() {
        assert_eq!(require_positive_amount(0), Err(StakingError::InvalidAmount));
        assert_eq!(require_positive_amount(1), Ok(1));
    }

    #[test]
    fn one_base_unit_scales_exactly() {
        assert_eq!(checked_scale_base_units(1), Ok(REWARD_PRECISION));
        assert_eq!(
            checked_scale_base_units(BASE_UNITS_PER_TOKEN),
            Ok(1_000_000_000_000_000)
        );
    }

    #[test]
    fn fractional_scaled_rewards_preserve_remainder_until_claimable() {
        let pending = REWARD_PRECISION + (REWARD_PRECISION / 2);

        assert_eq!(checked_claimable_base_units(pending), Ok(1));
        assert_eq!(checked_paid_scaled(1), Ok(REWARD_PRECISION));
        assert_eq!(
            checked_sub_scaled(pending, REWARD_PRECISION),
            Ok(500_000_000)
        );
    }

    #[test]
    fn maximum_safe_u64_values_can_scale_to_u128() {
        assert_eq!(
            checked_scale_base_units(u64::MAX),
            Ok(u128::from(u64::MAX) * REWARD_PRECISION)
        );
        assert_eq!(
            checked_mul_u64_to_u128(u64::MAX, u64::MAX),
            Ok(u128::from(u64::MAX) * u128::from(u64::MAX))
        );
    }

    #[test]
    fn emission_scaling_rejects_u128_overflow() {
        assert_eq!(
            checked_emission_scaled(u64::MAX, u64::MAX),
            Err(StakingError::ArithmeticOverflow)
        );
    }

    #[test]
    fn checked_addition_rejects_overflow() {
        assert_eq!(
            checked_add_u64(u64::MAX, 1),
            Err(StakingError::ArithmeticOverflow)
        );
        assert_eq!(
            checked_add_scaled(u128::MAX, 1),
            Err(StakingError::ArithmeticOverflow)
        );
    }

    #[test]
    fn checked_subtraction_rejects_underflow() {
        assert_eq!(
            checked_sub_u64(0, 1),
            Err(StakingError::ArithmeticUnderflow)
        );
        assert_eq!(
            checked_sub_scaled(0, 1),
            Err(StakingError::ArithmeticUnderflow)
        );
    }

    #[test]
    fn narrowing_rejects_values_above_u64_max() {
        assert_eq!(checked_u128_to_u64(u128::from(u64::MAX)), Ok(u64::MAX));
        assert_eq!(
            checked_u128_to_u64(u128::from(u64::MAX) + 1),
            Err(StakingError::ArithmeticOverflow)
        );
    }
}
