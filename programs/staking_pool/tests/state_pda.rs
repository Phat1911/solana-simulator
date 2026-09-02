use anchor_lang::prelude::{Pubkey, Result};
use anchor_lang::AnchorSerialize;
use staking_pool::state::{
    derive_pool_authority_pda, derive_pool_pda, derive_position_pda, derive_proposal_pda, Pool,
    Position, Proposal, ProposalAction, ANCHOR_DISCRIMINATOR_SIZE, POOL_AUTHORITY_SEED, POOL_SEED,
    POSITION_SEED, PROPOSAL_SEED,
};

fn key(byte: u8) -> Pubkey {
    Pubkey::from([byte; 32])
}

fn serialized_len<T: AnchorSerialize>(value: &T) -> Result<usize> {
    let mut bytes = Vec::new();
    value.serialize(&mut bytes)?;
    Ok(bytes.len())
}

#[test]
fn seed_prefixes_match_the_spec_exactly() {
    assert_eq!(POOL_SEED, b"pool");
    assert_eq!(POOL_AUTHORITY_SEED, b"pool-authority");
    assert_eq!(POSITION_SEED, b"position");
    assert_eq!(PROPOSAL_SEED, b"proposal");
}

#[test]
fn pool_pda_is_deterministic_and_uses_little_endian_pool_id() {
    let program_id = key(9);
    let initializer = key(1);
    let pool_id = 42_u64;
    let pool_id_bytes = pool_id.to_le_bytes();

    let expected = Pubkey::find_program_address(
        &[POOL_SEED, initializer.as_ref(), pool_id_bytes.as_ref()],
        &program_id,
    );

    assert_eq!(
        derive_pool_pda(&program_id, &initializer, pool_id),
        expected
    );
    assert_eq!(
        derive_pool_pda(&program_id, &initializer, pool_id),
        derive_pool_pda(&program_id, &initializer, pool_id)
    );
}

#[test]
fn pool_pda_separates_initializers_pool_ids_and_programs() {
    let program_id = key(9);
    let other_program_id = key(10);
    let initializer = key(1);
    let other_initializer = key(2);

    let pool = derive_pool_pda(&program_id, &initializer, 7).0;

    assert_ne!(pool, derive_pool_pda(&program_id, &other_initializer, 7).0);
    assert_ne!(pool, derive_pool_pda(&program_id, &initializer, 8).0);
    assert_ne!(pool, derive_pool_pda(&other_program_id, &initializer, 7).0);
}

#[test]
fn pool_authority_pda_is_bound_to_one_pool_and_has_no_data_layout() {
    let program_id = key(9);
    let pool = key(3);
    let other_pool = key(4);

    let expected = Pubkey::find_program_address(&[POOL_AUTHORITY_SEED, pool.as_ref()], &program_id);

    assert_eq!(derive_pool_authority_pda(&program_id, &pool), expected);
    assert_ne!(
        derive_pool_authority_pda(&program_id, &pool).0,
        derive_pool_authority_pda(&program_id, &other_pool).0
    );
}

#[test]
fn position_pda_is_canonical_per_pool_and_user_pair() {
    let program_id = key(9);
    let pool = key(3);
    let other_pool = key(4);
    let user = key(5);
    let other_user = key(6);

    let expected =
        Pubkey::find_program_address(&[POSITION_SEED, pool.as_ref(), user.as_ref()], &program_id);

    assert_eq!(derive_position_pda(&program_id, &pool, &user), expected);
    assert_ne!(
        derive_position_pda(&program_id, &pool, &user).0,
        derive_position_pda(&program_id, &other_pool, &user).0
    );
    assert_ne!(
        derive_position_pda(&program_id, &pool, &user).0,
        derive_position_pda(&program_id, &pool, &other_user).0
    );
}

#[test]
fn proposal_pda_is_canonical_per_pool_and_proposal_id() {
    let program_id = key(9);
    let pool = key(3);
    let other_pool = key(4);
    let proposal_id = 11_u64;
    let proposal_id_bytes = proposal_id.to_le_bytes();

    let expected = Pubkey::find_program_address(
        &[PROPOSAL_SEED, pool.as_ref(), proposal_id_bytes.as_ref()],
        &program_id,
    );

    assert_eq!(
        derive_proposal_pda(&program_id, &pool, proposal_id),
        expected
    );
    assert_ne!(
        derive_proposal_pda(&program_id, &pool, proposal_id).0,
        derive_proposal_pda(&program_id, &other_pool, proposal_id).0
    );
    assert_ne!(
        derive_proposal_pda(&program_id, &pool, proposal_id).0,
        derive_proposal_pda(&program_id, &pool, proposal_id + 1).0
    );
}

#[test]
fn account_sizes_are_fixed_and_include_anchor_discriminator() {
    assert_eq!(ANCHOR_DISCRIMINATOR_SIZE, 8);
    assert_eq!(Pool::LEN, 364);
    assert_eq!(Position::LEN, 106);
    assert_eq!(ProposalAction::MAX_SIZE, 65);
    assert_eq!(Proposal::LEN, 168);

    assert_eq!(Pool::SPACE, ANCHOR_DISCRIMINATOR_SIZE + Pool::LEN);
    assert_eq!(Position::SPACE, ANCHOR_DISCRIMINATOR_SIZE + Position::LEN);
    assert_eq!(Proposal::SPACE, ANCHOR_DISCRIMINATOR_SIZE + Proposal::LEN);
}

#[test]
fn proposal_action_variants_are_fixed_size_and_allowlisted() -> Result<()> {
    assert_eq!(ProposalAction::MAX_SIZE, 65);

    let set_reward_rate = ProposalAction::SetRewardRate { new_rate: 100 };
    let unpause_pool = ProposalAction::UnpausePool;
    let replace_admin = ProposalAction::ReplaceAdmin {
        old_admin: key(7),
        new_admin: key(8),
    };

    assert_eq!(serialized_len(&set_reward_rate)?, 9);
    assert_eq!(serialized_len(&unpause_pool)?, 1);
    assert_eq!(serialized_len(&replace_admin)?, ProposalAction::MAX_SIZE);

    Ok(())
}
