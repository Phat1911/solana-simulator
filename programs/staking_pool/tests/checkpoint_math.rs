//! Milestone 4: focused tests for pure global reward checkpoint math.

use staking_pool::constants::REWARD_PRECISION;
use staking_pool::error::StakingError;
use staking_pool::math::{checkpoint_pool_rewards, PoolCheckpointInput, PoolCheckpointOutput};

const P: u128 = REWARD_PRECISION;

fn base_checkpoint_input() -> PoolCheckpointInput {
    PoolCheckpointInput {
        paused: false,
        last_update_slot: 0,
        current_slot: 0,
        reward_rate_per_slot_base_units: 0,
        total_staked_base_units: 0,
        acc_reward_per_stake_scaled: 0,
        remaining_reward_budget_scaled: 0,
        allocated_liability_scaled: 0,
    }
}

#[test]
fn checkpoint_updates_global_accumulator_for_normal_accrual() {
    struct Case {
        name: &'static str,
        input: PoolCheckpointInput,
        expected: PoolCheckpointOutput,
    }

    let cases = [Case {
        name: "five slots at twenty base units across ten staked base units",
        input: PoolCheckpointInput {
            last_update_slot: 10,
            current_slot: 15,
            reward_rate_per_slot_base_units: 20,
            total_staked_base_units: 10,
            acc_reward_per_stake_scaled: 3 * P,
            remaining_reward_budget_scaled: 1_000 * P,
            allocated_liability_scaled: 7 * P,
            ..base_checkpoint_input()
        },
        expected: PoolCheckpointOutput {
            last_update_slot: 15,
            acc_reward_per_stake_scaled: 13 * P,
            remaining_reward_budget_scaled: 900 * P,
            allocated_liability_scaled: 107 * P,
            elapsed_slots: 5,
            emitted_scaled: 100 * P,
        },
    }];

    for case in cases {
        assert_eq!(
            checkpoint_pool_rewards(case.input),
            Ok(case.expected),
            "{}",
            case.name
        );
    }
}

#[test]
fn checkpoint_advances_empty_pool_without_consuming_budget() {
    let input = PoolCheckpointInput {
        last_update_slot: 100,
        current_slot: 125,
        reward_rate_per_slot_base_units: 50,
        acc_reward_per_stake_scaled: 11 * P,
        remaining_reward_budget_scaled: 300 * P,
        allocated_liability_scaled: 19 * P,
        ..base_checkpoint_input()
    };

    assert_eq!(
        checkpoint_pool_rewards(input),
        Ok(PoolCheckpointOutput {
            last_update_slot: 125,
            acc_reward_per_stake_scaled: 11 * P,
            remaining_reward_budget_scaled: 300 * P,
            allocated_liability_scaled: 19 * P,
            elapsed_slots: 25,
            emitted_scaled: 0,
        })
    );
}

#[test]
fn checkpoint_advances_paused_gap_without_accruing_rewards() {
    let input = PoolCheckpointInput {
        paused: true,
        last_update_slot: 200,
        current_slot: 260,
        reward_rate_per_slot_base_units: 100,
        total_staked_base_units: 25,
        acc_reward_per_stake_scaled: 2 * P,
        remaining_reward_budget_scaled: 500 * P,
        allocated_liability_scaled: 40 * P,
    };

    assert_eq!(
        checkpoint_pool_rewards(input),
        Ok(PoolCheckpointOutput {
            last_update_slot: 260,
            acc_reward_per_stake_scaled: 2 * P,
            remaining_reward_budget_scaled: 500 * P,
            allocated_liability_scaled: 40 * P,
            elapsed_slots: 60,
            emitted_scaled: 0,
        })
    );
}

#[test]
fn checkpoint_handles_exact_budget_exhaustion() {
    let input = PoolCheckpointInput {
        last_update_slot: 1_000,
        current_slot: 1_010,
        reward_rate_per_slot_base_units: 10,
        total_staked_base_units: 4,
        remaining_reward_budget_scaled: 100 * P,
        ..base_checkpoint_input()
    };

    assert_eq!(
        checkpoint_pool_rewards(input),
        Ok(PoolCheckpointOutput {
            last_update_slot: 1_010,
            acc_reward_per_stake_scaled: 25 * P,
            remaining_reward_budget_scaled: 0,
            allocated_liability_scaled: 100 * P,
            elapsed_slots: 10,
            emitted_scaled: 100 * P,
        })
    );
}

#[test]
fn checkpoint_caps_partial_final_emission_to_remaining_budget() {
    let input = PoolCheckpointInput {
        last_update_slot: 50,
        current_slot: 60,
        reward_rate_per_slot_base_units: 10,
        total_staked_base_units: 5,
        acc_reward_per_stake_scaled: P,
        remaining_reward_budget_scaled: 35 * P,
        allocated_liability_scaled: 9 * P,
        ..base_checkpoint_input()
    };

    assert_eq!(
        checkpoint_pool_rewards(input),
        Ok(PoolCheckpointOutput {
            last_update_slot: 60,
            acc_reward_per_stake_scaled: 8 * P,
            remaining_reward_budget_scaled: 0,
            allocated_liability_scaled: 44 * P,
            elapsed_slots: 10,
            emitted_scaled: 35 * P,
        })
    );
}

#[test]
fn checkpoint_preserves_rounding_remainder_in_budget() {
    let input = PoolCheckpointInput {
        last_update_slot: 7,
        current_slot: 8,
        reward_rate_per_slot_base_units: 10,
        total_staked_base_units: 3,
        remaining_reward_budget_scaled: 10 * P,
        ..base_checkpoint_input()
    };

    assert_eq!(
        checkpoint_pool_rewards(input),
        Ok(PoolCheckpointOutput {
            last_update_slot: 8,
            acc_reward_per_stake_scaled: 3_333_333_333,
            remaining_reward_budget_scaled: 1,
            allocated_liability_scaled: 9_999_999_999,
            elapsed_slots: 1,
            emitted_scaled: 9_999_999_999,
        })
    );
}

#[test]
fn checkpoint_rejects_backward_slots() {
    let input = PoolCheckpointInput {
        last_update_slot: 20,
        current_slot: 19,
        reward_rate_per_slot_base_units: 1,
        total_staked_base_units: 1,
        remaining_reward_budget_scaled: P,
        ..base_checkpoint_input()
    };

    assert_eq!(
        checkpoint_pool_rewards(input),
        Err(StakingError::ArithmeticUnderflow)
    );
}
