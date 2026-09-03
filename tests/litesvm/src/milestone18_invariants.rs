//! Milestone 18: cross-instruction solvency, conservation, and rollback scenarios.

#![allow(clippy::unwrap_used)]

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, Keypair, Pubkey, Signer};
use solana_program_pack::Pack;
use solana_sdk::transaction::Transaction;
use staking_pool::{
    constants::{ADMIN_COUNT, REWARD_PRECISION, TOKEN_DECIMALS},
    state::{
        derive_pool_authority_pda, derive_pool_pda, derive_position_pda, derive_proposal_pda, Pool,
        Position, ProposalAction,
    },
};

const POOL_ID: u64 = 18;
const MAX_REWARD_RATE_PER_SLOT: u64 = 10_000;
const INITIAL_STAKE_PER_USER: u64 = 10_000;
const INITIAL_REWARD_SUPPLY: u64 = 100_000;

fn program_bytes() -> &'static [u8] {
    include_bytes!("../../../target/deploy/staking_pool.so")
}

fn new_context() -> AnchorContext {
    AnchorLiteSVM::build_with_program(staking_pool::ID, program_bytes())
}

fn create_mint(ctx: &mut AnchorContext) -> Pubkey {
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
    mint.pubkey()
}

fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address_with_program_id(
        owner,
        mint,
        &spl_token::ID,
    )
}

fn create_ata(ctx: &mut AnchorContext, owner: Pubkey, mint: Pubkey) -> Pubkey {
    let payer_pubkey = ctx.payer().pubkey();
    let account = ata(&owner, &mint);
    let ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer_pubkey,
        &owner,
        &mint,
        &spl_token::ID,
    );
    let payer = ctx.payer();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer_pubkey),
        &[payer],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).unwrap();
    account
}

fn create_token_account(ctx: &mut AnchorContext, owner: Pubkey, mint: Pubkey) -> Pubkey {
    let token_account = Keypair::new();
    let payer_pubkey = ctx.payer().pubkey();
    let create_account = solana_system_interface::instruction::create_account(
        &payer_pubkey,
        &token_account.pubkey(),
        ctx.svm
            .minimum_balance_for_rent_exemption(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN as u64,
        &spl_token::ID,
    );
    let initialize_account = spl_token::instruction::initialize_account3(
        &spl_token::ID,
        &token_account.pubkey(),
        &mint,
        &owner,
    )
    .unwrap();
    let payer = ctx.payer();
    let tx = Transaction::new_signed_with_payer(
        &[create_account, initialize_account],
        Some(&payer_pubkey),
        &[payer, &token_account],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).unwrap();
    token_account.pubkey()
}

fn mint_to(ctx: &mut AnchorContext, mint: Pubkey, destination: Pubkey, amount: u64) {
    let payer = ctx.payer();
    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &destination,
        &payer.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).unwrap();
}

fn fund_sol(ctx: &mut AnchorContext, recipient: Pubkey) {
    let payer = ctx.payer();
    let ix =
        solana_system_interface::instruction::transfer(&payer.pubkey(), &recipient, 1_000_000_000);
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

fn read_position(ctx: &AnchorContext, position: Pubkey) -> Position {
    let account = ctx.svm.get_account(&position).unwrap();
    Position::try_deserialize(&mut account.data.as_slice()).unwrap()
}

fn read_token(ctx: &AnchorContext, token_account: Pubkey) -> u64 {
    let account = ctx.svm.get_account(&token_account).unwrap();
    spl_token::state::Account::unpack(&account.data)
        .unwrap()
        .amount
}

struct TestUser {
    signer: Keypair,
    stake_ata: Pubkey,
    reward_ata: Pubkey,
    position: Pubkey,
}

struct Fixture {
    ctx: AnchorContext,
    pool: Pubkey,
    pool_authority: Pubkey,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
    stake_vault: Pubkey,
    reward_vault: Pubkey,
    admins: [Keypair; ADMIN_COUNT],
    funder: Keypair,
    funder_reward_account: Pubkey,
    alice: TestUser,
    bob: TestUser,
}

#[derive(Clone, Copy)]
enum UserId {
    Alice,
    Bob,
}

impl Fixture {
    fn user(&self, user: UserId) -> &TestUser {
        match user {
            UserId::Alice => &self.alice,
            UserId::Bob => &self.bob,
        }
    }

    fn execute_user(
        &mut self,
        user: UserId,
        ix: anchor_litesvm::Instruction,
    ) -> anchor_litesvm::TransactionResult {
        let signer = self.user(user).signer.insecure_clone();
        self.ctx.svm.expire_blockhash();
        self.ctx.execute_instruction(ix, &[&signer]).unwrap()
    }

    fn execute_admin(
        &mut self,
        admin_index: usize,
        ix: anchor_litesvm::Instruction,
    ) -> anchor_litesvm::TransactionResult {
        let signer = self.admins[admin_index].insecure_clone();
        self.ctx.svm.expire_blockhash();
        self.ctx.execute_instruction(ix, &[&signer]).unwrap()
    }

    fn execute_funder(
        &mut self,
        ix: anchor_litesvm::Instruction,
    ) -> anchor_litesvm::TransactionResult {
        let signer = self.funder.insecure_clone();
        self.ctx.svm.expire_blockhash();
        self.ctx.execute_instruction(ix, &[&signer]).unwrap()
    }

    fn execute_payer(
        &mut self,
        ix: anchor_litesvm::Instruction,
    ) -> anchor_litesvm::TransactionResult {
        let payer = self.ctx.payer().insecure_clone();
        self.ctx.svm.expire_blockhash();
        self.ctx.execute_instruction(ix, &[&payer]).unwrap()
    }
}

fn setup() -> Fixture {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx);
    let reward_mint = create_mint(&mut ctx);
    let initializer = ctx.payer().pubkey();
    let (pool, _) = derive_pool_pda(&staking_pool::ID, &initializer, POOL_ID);
    let (pool_authority, _) = derive_pool_authority_pda(&staking_pool::ID, &pool);
    let stake_vault = ata(&pool_authority, &stake_mint);
    let reward_vault = ata(&pool_authority, &reward_mint);
    let admins = [Keypair::new(), Keypair::new(), Keypair::new()];
    for admin in &admins {
        fund_sol(&mut ctx, admin.pubkey());
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

    let funder = Keypair::new();
    let alice_signer = Keypair::new();
    let bob_signer = Keypair::new();
    for signer in [&funder, &alice_signer, &bob_signer] {
        fund_sol(&mut ctx, signer.pubkey());
    }

    let funder_reward_account = create_token_account(&mut ctx, funder.pubkey(), reward_mint);
    let alice = create_user(&mut ctx, pool, stake_mint, reward_mint, alice_signer);
    let bob = create_user(&mut ctx, pool, stake_mint, reward_mint, bob_signer);
    mint_to(
        &mut ctx,
        reward_mint,
        funder_reward_account,
        INITIAL_REWARD_SUPPLY,
    );
    mint_to(
        &mut ctx,
        stake_mint,
        alice.stake_ata,
        INITIAL_STAKE_PER_USER,
    );
    mint_to(&mut ctx, stake_mint, bob.stake_ata, INITIAL_STAKE_PER_USER);

    Fixture {
        ctx,
        pool,
        pool_authority,
        stake_mint,
        reward_mint,
        stake_vault,
        reward_vault,
        admins,
        funder,
        funder_reward_account,
        alice,
        bob,
    }
}

fn create_user(
    ctx: &mut AnchorContext,
    pool: Pubkey,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
    signer: Keypair,
) -> TestUser {
    let stake_ata = create_ata(ctx, signer.pubkey(), stake_mint);
    let reward_ata = create_ata(ctx, signer.pubkey(), reward_mint);
    let position = derive_position_pda(&staking_pool::ID, &pool, &signer.pubkey()).0;
    let open = anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::OpenPosition {
            user: signer.pubkey(),
            pool,
            position,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::OpenPosition {}.data(),
    };
    ctx.execute_instruction(open, &[&signer])
        .unwrap()
        .assert_success();
    TestUser {
        signer,
        stake_ata,
        reward_ata,
        position,
    }
}

fn fund_rewards_ix(fixture: &Fixture, amount: u64) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::FundRewards {
            source_authority: fixture.funder.pubkey(),
            pool: fixture.pool,
            source_reward_account: fixture.funder_reward_account,
            reward_mint: fixture.reward_mint,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::FundRewards { amount }.data(),
    }
}

fn stake_ix(fixture: &Fixture, user: UserId, amount: u64) -> anchor_litesvm::Instruction {
    let user = fixture.user(user);
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::Stake {
            user: user.signer.pubkey(),
            pool: fixture.pool,
            position: user.position,
            stake_mint: fixture.stake_mint,
            reward_mint: fixture.reward_mint,
            user_stake_account: user.stake_ata,
            stake_vault: fixture.stake_vault,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::Stake { amount }.data(),
    }
}

fn unstake_ix(fixture: &Fixture, user: UserId, amount: u64) -> anchor_litesvm::Instruction {
    let user = fixture.user(user);
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::Unstake {
            user: user.signer.pubkey(),
            pool: fixture.pool,
            pool_authority: fixture.pool_authority,
            position: user.position,
            stake_mint: fixture.stake_mint,
            reward_mint: fixture.reward_mint,
            user_stake_account: user.stake_ata,
            stake_vault: fixture.stake_vault,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::Unstake { amount }.data(),
    }
}

fn claim_ix(fixture: &Fixture, user: UserId) -> anchor_litesvm::Instruction {
    let user = fixture.user(user);
    claim_ix_with_accounts(
        fixture,
        user.signer.pubkey(),
        user.position,
        user.reward_ata,
        fixture.reward_vault,
        spl_token::ID,
    )
}

fn claim_ix_with_accounts(
    fixture: &Fixture,
    user: Pubkey,
    position: Pubkey,
    user_reward_account: Pubkey,
    reward_vault: Pubkey,
    token_program: Pubkey,
) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::ClaimRewards {
            user,
            pool: fixture.pool,
            pool_authority: fixture.pool_authority,
            position,
            reward_mint: fixture.reward_mint,
            reward_vault,
            user_reward_account,
            token_program,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::ClaimRewards {}.data(),
    }
}

fn pause_ix(fixture: &Fixture, admin: Pubkey) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::PausePool {
            admin,
            pool: fixture.pool,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::PausePool {}.data(),
    }
}

fn emergency_withdraw_ix(fixture: &Fixture, user: UserId) -> anchor_litesvm::Instruction {
    let user = fixture.user(user);
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::EmergencyWithdraw {
            user: user.signer.pubkey(),
            pool: fixture.pool,
            pool_authority: fixture.pool_authority,
            position: user.position,
            stake_mint: fixture.stake_mint,
            reward_mint: fixture.reward_mint,
            user_stake_account: user.stake_ata,
            stake_vault: fixture.stake_vault,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::EmergencyWithdraw {}.data(),
    }
}

fn proposal_pubkey(fixture: &Fixture, proposal_id: u64) -> Pubkey {
    derive_proposal_pda(&staking_pool::ID, &fixture.pool, proposal_id).0
}

fn create_proposal_ix(
    fixture: &Fixture,
    proposal_id: u64,
    action: ProposalAction,
) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::CreateProposal {
            creator: fixture.admins[0].pubkey(),
            pool: fixture.pool,
            proposal: proposal_pubkey(fixture, proposal_id),
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

fn approve_proposal_ix(fixture: &Fixture, proposal_id: u64) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::ApproveProposal {
            admin: fixture.admins[1].pubkey(),
            pool: fixture.pool,
            proposal: proposal_pubkey(fixture, proposal_id),
        }
        .to_account_metas(None),
        data: staking_pool::instruction::ApproveProposal {}.data(),
    }
}

fn execute_proposal_ix(fixture: &Fixture, proposal_id: u64) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::ExecuteProposal {
            pool: fixture.pool,
            proposal: proposal_pubkey(fixture, proposal_id),
        }
        .to_account_metas(None),
        data: staking_pool::instruction::ExecuteProposal {}.data(),
    }
}

fn execute_governance_action(fixture: &mut Fixture, action: ProposalAction) {
    let proposal_id = read_pool(&fixture.ctx, fixture.pool).next_proposal_id;
    let create = create_proposal_ix(fixture, proposal_id, action);
    fixture.execute_admin(0, create).assert_success();
    assert_invariants(fixture);
    let approve = approve_proposal_ix(fixture, proposal_id);
    fixture.execute_admin(1, approve).assert_success();
    assert_invariants(fixture);
    let execute = execute_proposal_ix(fixture, proposal_id);
    fixture.execute_payer(execute).assert_success();
    assert_invariants(fixture);
}

#[derive(Debug, PartialEq, Eq)]
struct ProtocolSnapshot {
    pool: Pool,
    alice_position: Position,
    bob_position: Position,
    stake_vault: u64,
    reward_vault: u64,
    alice_stake: u64,
    alice_reward: u64,
    bob_stake: u64,
    bob_reward: u64,
    funder_reward: u64,
}

fn snapshot(fixture: &Fixture) -> ProtocolSnapshot {
    ProtocolSnapshot {
        pool: read_pool(&fixture.ctx, fixture.pool),
        alice_position: read_position(&fixture.ctx, fixture.alice.position),
        bob_position: read_position(&fixture.ctx, fixture.bob.position),
        stake_vault: read_token(&fixture.ctx, fixture.stake_vault),
        reward_vault: read_token(&fixture.ctx, fixture.reward_vault),
        alice_stake: read_token(&fixture.ctx, fixture.alice.stake_ata),
        alice_reward: read_token(&fixture.ctx, fixture.alice.reward_ata),
        bob_stake: read_token(&fixture.ctx, fixture.bob.stake_ata),
        bob_reward: read_token(&fixture.ctx, fixture.bob.reward_ata),
        funder_reward: read_token(&fixture.ctx, fixture.funder_reward_account),
    }
}

fn assert_invariants(fixture: &Fixture) {
    let state = snapshot(fixture);
    let positions = [&state.alice_position, &state.bob_position];
    let total_position_stake: u64 = positions
        .iter()
        .map(|position| position.staked_amount)
        .sum();
    let aggregate_entitlement: u128 = positions
        .iter()
        .map(|position| {
            let accumulated = u128::from(position.staked_amount)
                .checked_mul(state.pool.acc_reward_per_stake_scaled)
                .unwrap();
            position
                .pending_reward_scaled
                .checked_add(
                    accumulated
                        .checked_sub(position.reward_debt_scaled)
                        .unwrap(),
                )
                .unwrap()
        })
        .sum();

    assert_eq!(state.pool.total_staked, total_position_stake);
    assert_eq!(state.stake_vault, state.pool.total_staked);
    assert_eq!(
        state.alice_stake + state.bob_stake + state.stake_vault,
        INITIAL_STAKE_PER_USER * 2
    );
    assert_eq!(state.pool.allocated_liability_scaled, aggregate_entitlement);
    assert_eq!(
        state.pool.remaining_reward_budget_scaled + state.pool.allocated_liability_scaled,
        u128::from(state.reward_vault) * REWARD_PRECISION
    );
    assert_eq!(
        state.funder_reward + state.reward_vault + state.alice_reward + state.bob_reward,
        INITIAL_REWARD_SUPPLY
    );
}

fn assert_failed_without_protocol_change(
    fixture: &Fixture,
    before: &ProtocolSnapshot,
    result: &anchor_litesvm::TransactionResult,
) {
    assert!(!result.is_success());
    assert_eq!(&snapshot(fixture), before);
    assert_invariants(fixture);
}

#[test]
fn multi_user_governance_pause_and_forfeiture_preserve_exact_accounting() {
    let mut fixture = setup();
    assert_invariants(&fixture);

    let fund = fund_rewards_ix(&fixture, 10_000);
    fixture.execute_funder(fund).assert_success();
    assert_invariants(&fixture);
    execute_governance_action(&mut fixture, ProposalAction::SetRewardRate { new_rate: 10 });
    execute_governance_action(&mut fixture, ProposalAction::UnpausePool);

    let stake = stake_ix(&fixture, UserId::Alice, 1_000);
    fixture.execute_user(UserId::Alice, stake).assert_success();
    assert_invariants(&fixture);
    let start_slot = read_pool(&fixture.ctx, fixture.pool).last_update_slot;
    fixture.ctx.svm.warp_to_slot(start_slot + 5);
    let stake = stake_ix(&fixture, UserId::Bob, 1_000);
    fixture.execute_user(UserId::Bob, stake).assert_success();
    assert_invariants(&fixture);
    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(
        pool.remaining_reward_budget_scaled,
        9_950 * REWARD_PRECISION
    );
    assert_eq!(pool.allocated_liability_scaled, 50 * REWARD_PRECISION);

    fixture.ctx.svm.warp_to_slot(pool.last_update_slot + 4);
    let claim = claim_ix(&fixture, UserId::Alice);
    fixture.execute_user(UserId::Alice, claim).assert_success();
    assert_invariants(&fixture);
    assert_eq!(read_token(&fixture.ctx, fixture.alice.reward_ata), 70);

    let unstake = unstake_ix(&fixture, UserId::Bob, 400);
    fixture.execute_user(UserId::Bob, unstake).assert_success();
    assert_invariants(&fixture);
    let before_rate_change = read_pool(&fixture.ctx, fixture.pool);
    fixture
        .ctx
        .svm
        .warp_to_slot(before_rate_change.last_update_slot + 3);
    execute_governance_action(&mut fixture, ProposalAction::SetRewardRate { new_rate: 20 });
    let after_rate_change = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(after_rate_change.reward_rate_per_slot, 20);
    assert_eq!(
        after_rate_change.remaining_reward_budget_scaled,
        9_880 * REWARD_PRECISION
    );
    assert_eq!(
        after_rate_change.allocated_liability_scaled,
        50 * REWARD_PRECISION
    );

    fixture
        .ctx
        .svm
        .warp_to_slot(after_rate_change.last_update_slot + 2);
    let pause = pause_ix(&fixture, fixture.admins[0].pubkey());
    fixture.execute_admin(0, pause).assert_success();
    assert_invariants(&fixture);

    let before_failed_claim = snapshot(&fixture);
    let claim = claim_ix(&fixture, UserId::Alice);
    let result = fixture.execute_user(UserId::Alice, claim);
    assert_failed_without_protocol_change(&fixture, &before_failed_claim, &result);

    let emergency = emergency_withdraw_ix(&fixture, UserId::Bob);
    fixture
        .execute_user(UserId::Bob, emergency)
        .assert_success();
    assert_invariants(&fixture);
    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(pool.total_staked, 1_000);
    assert_eq!(pool.remaining_reward_budget_scaled, 9_886_250_000_000);
    assert_eq!(pool.allocated_liability_scaled, 43_750_000_000);

    execute_governance_action(&mut fixture, ProposalAction::UnpausePool);
    let active = read_pool(&fixture.ctx, fixture.pool);
    fixture.ctx.svm.warp_to_slot(active.last_update_slot + 2);
    let unstake = unstake_ix(&fixture, UserId::Alice, 1_000);
    fixture
        .execute_user(UserId::Alice, unstake)
        .assert_success();
    assert_invariants(&fixture);
    let claim = claim_ix(&fixture, UserId::Alice);
    fixture.execute_user(UserId::Alice, claim).assert_success();
    assert_invariants(&fixture);
    assert_eq!(read_token(&fixture.ctx, fixture.alice.reward_ata), 153);

    let emergency = emergency_withdraw_ix(&fixture, UserId::Alice);
    fixture
        .execute_user(UserId::Alice, emergency)
        .assert_success();
    assert_invariants(&fixture);
    let final_pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(final_pool.total_staked, 0);
    assert_eq!(final_pool.allocated_liability_scaled, 0);
    assert_eq!(
        final_pool.remaining_reward_budget_scaled,
        u128::from(read_token(&fixture.ctx, fixture.reward_vault)) * REWARD_PRECISION
    );
}

#[test]
fn substituted_accounts_and_unauthorized_pause_roll_back_every_protocol_field() {
    let mut fixture = setup();
    let fund = fund_rewards_ix(&fixture, 1_000);
    fixture.execute_funder(fund).assert_success();
    execute_governance_action(&mut fixture, ProposalAction::SetRewardRate { new_rate: 5 });
    execute_governance_action(&mut fixture, ProposalAction::UnpausePool);
    let stake = stake_ix(&fixture, UserId::Alice, 500);
    fixture.execute_user(UserId::Alice, stake).assert_success();
    let current = read_pool(&fixture.ctx, fixture.pool);
    fixture.ctx.svm.warp_to_slot(current.last_update_slot + 3);

    let before = snapshot(&fixture);
    let wrong_vault = claim_ix_with_accounts(
        &fixture,
        fixture.alice.signer.pubkey(),
        fixture.alice.position,
        fixture.alice.reward_ata,
        fixture.bob.reward_ata,
        spl_token::ID,
    );
    let result = fixture.execute_user(UserId::Alice, wrong_vault);
    assert_failed_without_protocol_change(&fixture, &before, &result);

    let before = snapshot(&fixture);
    let stolen_position = claim_ix_with_accounts(
        &fixture,
        fixture.bob.signer.pubkey(),
        fixture.alice.position,
        fixture.bob.reward_ata,
        fixture.reward_vault,
        spl_token::ID,
    );
    let result = fixture.execute_user(UserId::Bob, stolen_position);
    assert_failed_without_protocol_change(&fixture, &before, &result);

    let attacker = Keypair::new();
    fund_sol(&mut fixture.ctx, attacker.pubkey());
    let before = snapshot(&fixture);
    let unauthorized_pause = pause_ix(&fixture, attacker.pubkey());
    let result = fixture
        .ctx
        .execute_instruction(unauthorized_pause, &[&attacker])
        .unwrap();
    assert_failed_without_protocol_change(&fixture, &before, &result);

    let before = snapshot(&fixture);
    let wrong_program = claim_ix_with_accounts(
        &fixture,
        fixture.alice.signer.pubkey(),
        fixture.alice.position,
        fixture.alice.reward_ata,
        fixture.reward_vault,
        solana_sdk::system_program::ID,
    );
    let result = fixture.execute_user(UserId::Alice, wrong_program);
    assert_failed_without_protocol_change(&fixture, &before, &result);
}

struct DeterministicRng(u64);

impl DeterministicRng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn run_recorded_sequence(seed: u64) {
    let mut fixture = setup();
    let fund = fund_rewards_ix(&fixture, 5_000);
    fixture.execute_funder(fund).assert_success();
    execute_governance_action(&mut fixture, ProposalAction::SetRewardRate { new_rate: 7 });
    execute_governance_action(&mut fixture, ProposalAction::UnpausePool);
    for user in [UserId::Alice, UserId::Bob] {
        let stake = stake_ix(&fixture, user, 500);
        fixture.execute_user(user, stake).assert_success();
        assert_invariants(&fixture);
    }

    let mut rng = DeterministicRng(seed);
    for _step in 0..16 {
        match rng.next() % 5 {
            0 => {
                let pool = read_pool(&fixture.ctx, fixture.pool);
                fixture
                    .ctx
                    .svm
                    .warp_to_slot(pool.last_update_slot + 1 + rng.next() % 4);
                assert_invariants(&fixture);
            }
            1 => {
                let user = if rng.next().is_multiple_of(2) {
                    UserId::Alice
                } else {
                    UserId::Bob
                };
                let amount = 1 + rng.next() % 25;
                let before = snapshot(&fixture);
                let ix = stake_ix(&fixture, user, amount);
                let result = fixture.execute_user(user, ix);
                if result.is_success() {
                    assert_invariants(&fixture);
                } else {
                    assert_failed_without_protocol_change(&fixture, &before, &result);
                }
            }
            2 => {
                let user = if rng.next().is_multiple_of(2) {
                    UserId::Alice
                } else {
                    UserId::Bob
                };
                let staked = read_position(&fixture.ctx, fixture.user(user).position).staked_amount;
                if staked > 0 {
                    let amount = 1 + rng.next() % staked.min(25);
                    let ix = unstake_ix(&fixture, user, amount);
                    fixture.execute_user(user, ix).assert_success();
                    assert_invariants(&fixture);
                }
            }
            3 => {
                let user = if rng.next().is_multiple_of(2) {
                    UserId::Alice
                } else {
                    UserId::Bob
                };
                let before = snapshot(&fixture);
                let ix = claim_ix(&fixture, user);
                let result = fixture.execute_user(user, ix);
                if result.is_success() {
                    assert_invariants(&fixture);
                } else {
                    assert_failed_without_protocol_change(&fixture, &before, &result);
                }
            }
            _ => {
                let amount = 1 + rng.next() % 20;
                let ix = fund_rewards_ix(&fixture, amount);
                fixture.execute_funder(ix).assert_success();
                assert_invariants(&fixture);
            }
        }
    }

    let pause = pause_ix(&fixture, fixture.admins[2].pubkey());
    fixture.execute_admin(2, pause).assert_success();
    assert_invariants(&fixture);
    for user in [UserId::Alice, UserId::Bob] {
        let position = read_position(&fixture.ctx, fixture.user(user).position);
        if position.staked_amount > 0 || position.pending_reward_scaled > 0 {
            let ix = emergency_withdraw_ix(&fixture, user);
            fixture.execute_user(user, ix).assert_success();
            assert_invariants(&fixture);
        }
    }
}

#[test]
fn recorded_state_machine_seeds_preserve_invariants() {
    // Milestone 18: fixed seeds make every generated operation sequence replayable.
    for seed in [0x18_u64, 0x5eed_u64, 0xc0ffee_u64] {
        run_recorded_sequence(seed);
    }
}
