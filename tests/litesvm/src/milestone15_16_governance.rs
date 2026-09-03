//! Milestones 15-16: LiteSVM coverage for proposal governance and execution.

#![allow(clippy::unwrap_used)]

use anchor_lang::{AccountDeserialize, AccountSerialize, InstructionData, ToAccountMetas};
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, Keypair, Pubkey, Signer};
use solana_program_pack::Pack;
use solana_sdk::{account::Account, transaction::Transaction};
use staking_pool::{
    constants::{ADMIN_COUNT, PROPOSAL_TTL_SLOTS, REWARD_PRECISION, TOKEN_DECIMALS},
    state::{
        derive_pool_authority_pda, derive_pool_pda, derive_proposal_pda, Pool, Proposal,
        ProposalAction,
    },
};

const POOL_ID: u64 = 16;
const MAX_REWARD_RATE_PER_SLOT: u64 = 10_000;

fn program_bytes() -> &'static [u8] {
    include_bytes!("../../../target/deploy/staking_pool.so")
}

fn new_context() -> AnchorContext {
    AnchorLiteSVM::build_with_program(staking_pool::ID, program_bytes())
}

fn create_mint(ctx: &mut AnchorContext) -> Keypair {
    let mint = Keypair::new();
    let payer_pubkey = ctx.payer().pubkey();
    let create_account = solana_system_interface::instruction::create_account(
        &payer_pubkey,
        &mint.pubkey(),
        ctx.svm
            .minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN as u64,
        &spl_token::ID,
    );
    let initialize_mint = spl_token::instruction::initialize_mint2(
        &spl_token::ID,
        &mint.pubkey(),
        &payer_pubkey,
        None,
        TOKEN_DECIMALS,
    )
    .unwrap();
    let payer = ctx.payer();
    let tx = Transaction::new_signed_with_payer(
        &[create_account, initialize_mint],
        Some(&payer_pubkey),
        &[payer, &mint],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).unwrap();
    mint
}

fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address_with_program_id(
        owner,
        mint,
        &spl_token::ID,
    )
}

fn fund_sol(ctx: &mut AnchorContext, user: &Keypair) {
    let payer = ctx.payer();
    let ix = solana_system_interface::instruction::transfer(
        &payer.pubkey(),
        &user.pubkey(),
        1_000_000_000,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).unwrap();
}

fn read_pool(ctx: &AnchorContext, pool: Pubkey) -> Pool {
    let account = ctx.svm.get_account(&pool).unwrap();
    Pool::try_deserialize(&mut account.data.as_slice()).unwrap()
}

fn write_pool(ctx: &mut AnchorContext, pool_pubkey: Pubkey, pool: &Pool) {
    let mut account = ctx.svm.get_account(&pool_pubkey).unwrap();
    account.data.clear();
    pool.try_serialize(&mut account.data).unwrap();
    ctx.svm
        .set_account(
            pool_pubkey,
            Account {
                lamports: account.lamports,
                data: account.data,
                owner: account.owner,
                executable: account.executable,
                rent_epoch: account.rent_epoch,
            },
        )
        .unwrap();
}

fn read_proposal(ctx: &AnchorContext, proposal: Pubkey) -> Proposal {
    let account = ctx.svm.get_account(&proposal).unwrap();
    Proposal::try_deserialize(&mut account.data.as_slice()).unwrap()
}

struct Fixture {
    ctx: AnchorContext,
    pool: Pubkey,
    admins: [Keypair; ADMIN_COUNT],
}

fn setup() -> Fixture {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx).pubkey();
    let reward_mint = create_mint(&mut ctx).pubkey();
    let initializer = ctx.payer().pubkey();
    let (pool, _) = derive_pool_pda(&staking_pool::ID, &initializer, POOL_ID);
    let (pool_authority, _) = derive_pool_authority_pda(&staking_pool::ID, &pool);
    let stake_vault = ata(&pool_authority, &stake_mint);
    let reward_vault = ata(&pool_authority, &reward_mint);
    let admins = [Keypair::new(), Keypair::new(), Keypair::new()];
    for admin in &admins {
        fund_sol(&mut ctx, admin);
    }
    let admin_pubkeys = admins.each_ref().map(Keypair::pubkey);

    let initialize = anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::InitializePool {
            initializer,
            pool,
            pool_authority,
            stake_mint,
            reward_mint,
            stake_vault,
            reward_vault,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: solana_sdk::system_program::ID,
            rent: solana_sdk::sysvar::rent::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::InitializePool {
            pool_id: POOL_ID,
            admins: admin_pubkeys,
            max_reward_rate_per_slot: MAX_REWARD_RATE_PER_SLOT,
        }
        .data(),
    };
    let payer = ctx.payer().insecure_clone();
    ctx.execute_instruction(initialize, &[&payer])
        .unwrap()
        .assert_success();

    Fixture { ctx, pool, admins }
}

fn proposal_pubkey(pool: Pubkey, proposal_id: u64) -> Pubkey {
    derive_proposal_pda(&staking_pool::ID, &pool, proposal_id).0
}

fn create_proposal_ix(
    fixture: &Fixture,
    creator_index: usize,
    proposal_id: u64,
    action: ProposalAction,
) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::CreateProposal {
            creator: fixture.admins[creator_index].pubkey(),
            pool: fixture.pool,
            proposal: proposal_pubkey(fixture.pool, proposal_id),
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::CreateProposal {
            proposal_id,
            action,
        }
        .data(),
    }
}

fn approve_ix(
    fixture: &Fixture,
    admin_index: usize,
    proposal_id: u64,
) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::ApproveProposal {
            admin: fixture.admins[admin_index].pubkey(),
            pool: fixture.pool,
            proposal: proposal_pubkey(fixture.pool, proposal_id),
        }
        .to_account_metas(None),
        data: staking_pool::instruction::ApproveProposal {}.data(),
    }
}

fn execute_ix(fixture: &Fixture, proposal_id: u64) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::ExecuteProposal {
            pool: fixture.pool,
            proposal: proposal_pubkey(fixture.pool, proposal_id),
        }
        .to_account_metas(None),
        data: staking_pool::instruction::ExecuteProposal {}.data(),
    }
}

fn close_ix(fixture: &Fixture, proposal_id: u64, creator: Pubkey) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::CloseProposal {
            payer: fixture.admins[2].pubkey(),
            pool: fixture.pool,
            proposal: proposal_pubkey(fixture.pool, proposal_id),
            creator,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::CloseProposal {}.data(),
    }
}

fn execute_admin(
    fixture: &mut Fixture,
    ix: anchor_litesvm::Instruction,
    admin_index: usize,
) -> anchor_litesvm::TransactionResult {
    fixture
        .ctx
        .execute_instruction(ix, &[&fixture.admins[admin_index]])
        .unwrap()
}

fn execute_payer(
    fixture: &mut Fixture,
    ix: anchor_litesvm::Instruction,
) -> anchor_litesvm::TransactionResult {
    let payer = fixture.ctx.payer().insecure_clone();
    fixture.ctx.execute_instruction(ix, &[&payer]).unwrap()
}

#[test]
fn create_proposal_records_creator_approval_and_increments_sequence() {
    let mut fixture = setup();
    let created_slot = read_pool(&fixture.ctx, fixture.pool).last_update_slot;
    let action = ProposalAction::SetRewardRate { new_rate: 7 };

    {
        let ix = create_proposal_ix(&fixture, 0, 0, action.clone());
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();

    let pool = read_pool(&fixture.ctx, fixture.pool);
    let proposal = read_proposal(&fixture.ctx, proposal_pubkey(fixture.pool, 0));
    assert_eq!(pool.next_proposal_id, 1);
    assert_eq!(proposal.pool, fixture.pool);
    assert_eq!(proposal.proposal_id, 0);
    assert_eq!(proposal.creator, fixture.admins[0].pubkey());
    assert_eq!(proposal.admin_epoch, 0);
    assert_eq!(proposal.action, action);
    assert_eq!(proposal.approvals, [true, false, false]);
    assert_eq!(proposal.approval_count, 1);
    assert!(proposal.expires_at_slot >= created_slot + PROPOSAL_TTL_SLOTS);
    assert!(!proposal.executed);
}

#[test]
fn create_proposal_rejects_wrong_id_bad_rate_and_bad_admin_replacement() {
    let mut fixture = setup();
    let wrong_id = {
        let ix = create_proposal_ix(&fixture, 0, 1, ProposalAction::UnpausePool);
        execute_admin(&mut fixture, ix, 0)
    };
    let bad_rate = {
        let ix = create_proposal_ix(
            &fixture,
            0,
            0,
            ProposalAction::SetRewardRate {
                new_rate: MAX_REWARD_RATE_PER_SLOT + 1,
            },
        );
        execute_admin(&mut fixture, ix, 0)
    };
    let bad_replacement = {
        let ix = create_proposal_ix(
            &fixture,
            0,
            0,
            ProposalAction::ReplaceAdmin {
                old_admin: Pubkey::new_unique(),
                new_admin: fixture.admins[1].pubkey(),
            },
        );
        execute_admin(&mut fixture, ix, 0)
    };

    assert!(!wrong_id.is_success());
    assert!(!bad_rate.is_success());
    assert!(!bad_replacement.is_success());
    assert_eq!(read_pool(&fixture.ctx, fixture.pool).next_proposal_id, 0);
}

#[test]
fn approve_proposal_counts_distinct_current_admins_only() {
    let mut fixture = setup();
    {
        let ix = create_proposal_ix(&fixture, 0, 0, ProposalAction::UnpausePool);
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();
    let duplicate = {
        let ix = approve_ix(&fixture, 0, 0);
        execute_admin(&mut fixture, ix, 0)
    };
    let second = {
        let ix = approve_ix(&fixture, 1, 0);
        execute_admin(&mut fixture, ix, 1)
    };

    assert!(!duplicate.is_success());
    assert!(second.is_success());
    let proposal = read_proposal(&fixture.ctx, proposal_pubkey(fixture.pool, 0));
    assert_eq!(proposal.approvals, [true, true, false]);
    assert_eq!(proposal.approval_count, 2);
}

#[test]
fn execute_set_rate_requires_threshold_and_checkpoints_old_rate() {
    let mut fixture = setup();
    let mut pool = read_pool(&fixture.ctx, fixture.pool);
    pool.paused = false;
    pool.total_staked = 1_000;
    pool.reward_rate_per_slot = 5;
    pool.remaining_reward_budget_scaled = 1_000 * REWARD_PRECISION;
    pool.last_update_slot = fixture
        .ctx
        .svm
        .get_sysvar::<solana_sdk::clock::Clock>()
        .slot;
    write_pool(&mut fixture.ctx, fixture.pool, &pool);
    {
        let ix = create_proposal_ix(
            &fixture,
            0,
            0,
            ProposalAction::SetRewardRate { new_rate: 9 },
        );
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();
    let start_slot = read_pool(&fixture.ctx, fixture.pool).last_update_slot;
    fixture.ctx.svm.warp_to_slot(start_slot + 4);
    let too_early = {
        let ix = execute_ix(&fixture, 0);
        execute_admin(&mut fixture, ix, 2)
    };
    assert!(!too_early.is_success());
    fixture.ctx.svm.warp_to_slot(start_slot + 5);

    {
        let ix = approve_ix(&fixture, 1, 0);
        execute_admin(&mut fixture, ix, 1)
    }
    .assert_success();
    {
        let ix = execute_ix(&fixture, 0);
        execute_payer(&mut fixture, ix)
    }
    .assert_success();

    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(pool.reward_rate_per_slot, 9);
    assert_eq!(pool.allocated_liability_scaled, 25 * REWARD_PRECISION);
    assert_eq!(pool.remaining_reward_budget_scaled, 975 * REWARD_PRECISION);
    assert_eq!(pool.last_update_slot, start_slot + 5);
    let replay = {
        let ix = execute_ix(&fixture, 0);
        execute_payer(&mut fixture, ix)
    };
    assert!(!replay.is_success());
}

#[test]
fn execute_unpause_sets_current_slot_without_backpay() {
    let mut fixture = setup();
    {
        let ix = create_proposal_ix(&fixture, 0, 0, ProposalAction::UnpausePool);
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();
    {
        let ix = approve_ix(&fixture, 1, 0);
        execute_admin(&mut fixture, ix, 1)
    }
    .assert_success();
    let before = read_pool(&fixture.ctx, fixture.pool);
    fixture.ctx.svm.warp_to_slot(before.last_update_slot + 10);
    {
        let ix = execute_ix(&fixture, 0);
        execute_payer(&mut fixture, ix)
    }
    .assert_success();

    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert!(!pool.paused);
    assert_eq!(pool.last_update_slot, before.last_update_slot + 10);
    assert_eq!(pool.allocated_liability_scaled, 0);
}

#[test]
fn execute_replace_admin_rotates_admin_and_stales_old_epoch_proposals() {
    let mut fixture = setup();
    let new_admin = Keypair::new();
    fund_sol(&mut fixture.ctx, &new_admin);
    {
        let ix = create_proposal_ix(&fixture, 0, 0, ProposalAction::UnpausePool);
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();
    {
        let ix = create_proposal_ix(
            &fixture,
            0,
            1,
            ProposalAction::ReplaceAdmin {
                old_admin: fixture.admins[2].pubkey(),
                new_admin: new_admin.pubkey(),
            },
        );
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();
    {
        let ix = approve_ix(&fixture, 1, 1);
        execute_admin(&mut fixture, ix, 1)
    }
    .assert_success();
    {
        let ix = execute_ix(&fixture, 1);
        execute_payer(&mut fixture, ix)
    }
    .assert_success();

    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(pool.admins[2], new_admin.pubkey());
    assert_eq!(pool.admin_epoch, 1);
    let stale_execute = {
        let ix = execute_ix(&fixture, 0);
        execute_payer(&mut fixture, ix)
    };
    assert!(!stale_execute.is_success());
}

#[test]
fn close_proposal_returns_rent_to_creator_after_execution_or_expiry() {
    let mut fixture = setup();
    let creator = fixture.admins[0].pubkey();
    {
        let ix = create_proposal_ix(&fixture, 0, 0, ProposalAction::UnpausePool);
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();
    let before_creator_lamports = fixture.ctx.svm.get_account(&creator).unwrap().lamports;
    {
        let ix = approve_ix(&fixture, 1, 0);
        execute_admin(&mut fixture, ix, 1)
    }
    .assert_success();
    {
        let ix = execute_ix(&fixture, 0);
        execute_payer(&mut fixture, ix)
    }
    .assert_success();
    {
        let ix = close_ix(&fixture, 0, creator);
        execute_admin(&mut fixture, ix, 2)
    }
    .assert_success();
    let closed_proposal = fixture
        .ctx
        .svm
        .get_account(&proposal_pubkey(fixture.pool, 0))
        .unwrap();
    assert_eq!(closed_proposal.lamports, 0);
    assert!(closed_proposal.data.iter().all(|byte| *byte == 0));
    assert!(fixture.ctx.svm.get_account(&creator).unwrap().lamports > before_creator_lamports);

    {
        let ix = create_proposal_ix(&fixture, 0, 1, ProposalAction::UnpausePool);
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();
    let proposal = read_proposal(&fixture.ctx, proposal_pubkey(fixture.pool, 1));
    fixture.ctx.svm.warp_to_slot(proposal.expires_at_slot + 1);
    {
        let ix = close_ix(&fixture, 1, creator);
        execute_admin(&mut fixture, ix, 2)
    }
    .assert_success();
    let closed_proposal = fixture
        .ctx
        .svm
        .get_account(&proposal_pubkey(fixture.pool, 1))
        .unwrap();
    assert_eq!(closed_proposal.lamports, 0);
    assert!(closed_proposal.data.iter().all(|byte| *byte == 0));
}
