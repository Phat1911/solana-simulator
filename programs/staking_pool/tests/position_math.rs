//! Milestone 5: focused tests for pure position settlement, claim, and forfeiture math.

use staking_pool::constants::REWARD_PRECISION;
use staking_pool::error::StakingError;
use staking_pool::math::{
    claim_position_rewards, forfeit_position_rewards, reset_position_reward_debt,
    settle_position_rewards, ClaimPositionInput, ClaimPositionOutput, ForfeitPositionInput,
    ForfeitPositionOutput, PositionSettlementInput, PositionSettlementOutput,
};

const P: u128 = REWARD_PRECISION;
type TestResult = Result<(), StakingError>;

fn empty_position() -> PositionSettlementInput {
    PositionSettlementInput {
        staked_amount_base_units: 0,
        reward_debt_scaled: 0,
        pending_reward_scaled: 0,
        acc_reward_per_stake_scaled: 0,
    }
}

#[test]
fn settlement_distributes_to_multiple_users_without_order_dependence() -> TestResult {
    let alice = PositionSettlementInput {
        staked_amount_base_units: 10,
        reward_debt_scaled: 0,
        pending_reward_scaled: 0,
        acc_reward_per_stake_scaled: 7 * P,
    };
    let bob = PositionSettlementInput {
        staked_amount_base_units: 30,
        reward_debt_scaled: 0,
        pending_reward_scaled: 0,
        acc_reward_per_stake_scaled: 7 * P,
    };

    let alice_first = settle_position_rewards(alice)?;
    let bob_second = settle_position_rewards(bob)?;
    let bob_first = settle_position_rewards(bob)?;
    let alice_second = settle_position_rewards(alice)?;

    assert_eq!(alice_first.pending_reward_scaled, 70 * P);
    assert_eq!(bob_second.pending_reward_scaled, 210 * P);
    assert_eq!(bob_first, bob_second);
    assert_eq!(alice_second, alice_first);
    Ok(())
}

#[test]
fn staggered_entry_resets_reward_debt_so_new_stake_gets_no_backpay() -> TestResult {
    let alice_after_first_checkpoint = settle_position_rewards(PositionSettlementInput {
        staked_amount_base_units: 10,
        reward_debt_scaled: 0,
        pending_reward_scaled: 0,
        acc_reward_per_stake_scaled: 5 * P,
    })?;

    let bob_reward_debt = reset_position_reward_debt(20, 5 * P)?;

    let alice_after_second_checkpoint = settle_position_rewards(PositionSettlementInput {
        staked_amount_base_units: 10,
        reward_debt_scaled: alice_after_first_checkpoint.reward_debt_scaled,
        pending_reward_scaled: alice_after_first_checkpoint.pending_reward_scaled,
        acc_reward_per_stake_scaled: 8 * P,
    })?;
    let bob_after_second_checkpoint = settle_position_rewards(PositionSettlementInput {
        staked_amount_base_units: 20,
        reward_debt_scaled: bob_reward_debt,
        pending_reward_scaled: 0,
        acc_reward_per_stake_scaled: 8 * P,
    })?;

    assert_eq!(alice_after_first_checkpoint.pending_reward_scaled, 50 * P);
    assert_eq!(bob_reward_debt, 100 * P);
    assert_eq!(alice_after_second_checkpoint.pending_reward_scaled, 80 * P);
    assert_eq!(bob_after_second_checkpoint.pending_reward_scaled, 60 * P);
    Ok(())
}

#[test]
fn repeated_settlement_at_same_accumulator_creates_no_extra_reward() -> TestResult {
    let first = settle_position_rewards(PositionSettlementInput {
        staked_amount_base_units: 12,
        reward_debt_scaled: 24 * P,
        pending_reward_scaled: 3 * P,
        acc_reward_per_stake_scaled: 5 * P,
    })?;
    let second = settle_position_rewards(PositionSettlementInput {
        staked_amount_base_units: 12,
        reward_debt_scaled: first.reward_debt_scaled,
        pending_reward_scaled: first.pending_reward_scaled,
        acc_reward_per_stake_scaled: 5 * P,
    });

    assert_eq!(
        first,
        PositionSettlementOutput {
            accrued_scaled: 60 * P,
            reward_debt_scaled: 60 * P,
            pending_reward_scaled: 39 * P,
            newly_earned_scaled: 36 * P,
        }
    );
    assert_eq!(
        second,
        Ok(PositionSettlementOutput {
            accrued_scaled: 60 * P,
            reward_debt_scaled: 60 * P,
            pending_reward_scaled: 39 * P,
            newly_earned_scaled: 0,
        })
    );
    Ok(())
}

#[test]
fn claim_pays_whole_base_units_and_preserves_scaled_remainder() {
    let input = ClaimPositionInput {
        pending_reward_scaled: (3 * P) + 250,
        allocated_liability_scaled: 10 * P,
    };

    assert_eq!(
        claim_position_rewards(input),
        Ok(ClaimPositionOutput {
            claimed_base_units: 3,
            paid_scaled: 3 * P,
            pending_reward_scaled: 250,
            allocated_liability_scaled: 7 * P,
        })
    );
}

#[test]
fn post_claim_accrual_adds_new_rewards_without_losing_fractional_remainder() -> TestResult {
    let claimed = claim_position_rewards(ClaimPositionInput {
        pending_reward_scaled: (4 * P) + 123,
        allocated_liability_scaled: 20 * P,
    })?;

    let settled = settle_position_rewards(PositionSettlementInput {
        staked_amount_base_units: 5,
        reward_debt_scaled: 10 * P,
        pending_reward_scaled: claimed.pending_reward_scaled,
        acc_reward_per_stake_scaled: 6 * P,
    });

    assert_eq!(
        settled,
        Ok(PositionSettlementOutput {
            accrued_scaled: 30 * P,
            reward_debt_scaled: 30 * P,
            pending_reward_scaled: (20 * P) + 123,
            newly_earned_scaled: 20 * P,
        })
    );
    Ok(())
}

#[test]
fn forfeiture_moves_exact_pending_liability_back_to_unallocated_budget() -> TestResult {
    let input = ForfeitPositionInput {
        pending_reward_scaled: (9 * P) + 777,
        remaining_reward_budget_scaled: 41 * P,
        allocated_liability_scaled: (30 * P) + 777,
    };

    let before_total = input.remaining_reward_budget_scaled + input.allocated_liability_scaled;
    let output = forfeit_position_rewards(input)?;
    let after_total = output.remaining_reward_budget_scaled + output.allocated_liability_scaled;

    assert_eq!(
        output,
        ForfeitPositionOutput {
            forfeited_scaled: (9 * P) + 777,
            pending_reward_scaled: 0,
            remaining_reward_budget_scaled: (50 * P) + 777,
            allocated_liability_scaled: 21 * P,
        }
    );
    assert_eq!(after_total, before_total);
    Ok(())
}

#[test]
fn zero_whole_token_claim_returns_nothing_to_claim_and_preserves_liability() {
    let input = ClaimPositionInput {
        pending_reward_scaled: P - 1,
        allocated_liability_scaled: 5 * P,
    };

    assert_eq!(
        claim_position_rewards(input),
        Err(StakingError::NothingToClaim)
    );
}

#[test]
fn settlement_rejects_reward_debt_above_current_accrued_amount() {
    let input = PositionSettlementInput {
        staked_amount_base_units: 2,
        reward_debt_scaled: (7 * P) + 1,
        pending_reward_scaled: 0,
        acc_reward_per_stake_scaled: 3 * P,
    };

    assert_eq!(
        settle_position_rewards(input),
        Err(StakingError::ArithmeticUnderflow)
    );
}

#[test]
fn settlement_rejects_u128_overflow_when_multiplying_stake_by_accumulator() {
    let input = PositionSettlementInput {
        staked_amount_base_units: u64::MAX,
        reward_debt_scaled: 0,
        pending_reward_scaled: 0,
        acc_reward_per_stake_scaled: u128::MAX,
    };

    assert_eq!(
        settle_position_rewards(input),
        Err(StakingError::ArithmeticOverflow)
    );
}

#[test]
fn settlement_rejects_pending_reward_overflow() {
    let input = PositionSettlementInput {
        staked_amount_base_units: 1,
        reward_debt_scaled: 0,
        pending_reward_scaled: u128::MAX,
        acc_reward_per_stake_scaled: 1,
    };

    assert_eq!(
        settle_position_rewards(input),
        Err(StakingError::ArithmeticOverflow)
    );
}

#[test]
fn reset_reward_debt_rejects_u128_overflow() {
    assert_eq!(
        reset_position_reward_debt(u64::MAX, u128::MAX),
        Err(StakingError::ArithmeticOverflow)
    );
}

#[test]
fn claim_rejects_paid_amount_above_allocated_liability() {
    let input = ClaimPositionInput {
        pending_reward_scaled: 8 * P,
        allocated_liability_scaled: 7 * P,
    };

    assert_eq!(
        claim_position_rewards(input),
        Err(StakingError::ArithmeticUnderflow)
    );
}

#[test]
fn claim_rejects_claimable_amount_that_cannot_fit_in_u64() {
    let input = ClaimPositionInput {
        pending_reward_scaled: (u128::from(u64::MAX) + 1) * P,
        allocated_liability_scaled: u128::MAX,
    };

    assert_eq!(
        claim_position_rewards(input),
        Err(StakingError::ArithmeticOverflow)
    );
}

#[test]
fn forfeiture_rejects_pending_reward_above_allocated_liability() {
    let input = ForfeitPositionInput {
        pending_reward_scaled: 11 * P,
        remaining_reward_budget_scaled: 0,
        allocated_liability_scaled: 10 * P,
    };

    assert_eq!(
        forfeit_position_rewards(input),
        Err(StakingError::ArithmeticUnderflow)
    );
}

#[test]
fn forfeiture_rejects_budget_overflow_when_returning_pending_reward() {
    let input = ForfeitPositionInput {
        pending_reward_scaled: 1,
        remaining_reward_budget_scaled: u128::MAX,
        allocated_liability_scaled: 1,
    };

    assert_eq!(
        forfeit_position_rewards(input),
        Err(StakingError::ArithmeticOverflow)
    );
}

#[test]
fn empty_position_settlement_is_a_no_op() {
    assert_eq!(
        settle_position_rewards(empty_position()),
        Ok(PositionSettlementOutput {
            accrued_scaled: 0,
            reward_debt_scaled: 0,
            pending_reward_scaled: 0,
            newly_earned_scaled: 0,
        })
    );
}
