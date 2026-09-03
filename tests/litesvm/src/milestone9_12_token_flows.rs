//! Milestones 9-14: LiteSVM coverage for token flows, pause, and emergency withdrawal.

#![allow(clippy::unwrap_used)]

use anchor_lang::{AccountDeserialize, AccountSerialize, InstructionData, ToAccountMetas};
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, Keypair, Pubkey, Signer};
use solana_program_pack::Pack;
use solana_sdk::{account::Account, transaction::Transaction};
use staking_pool::{
    constants::{ADMIN_COUNT, REWARD_PRECISION, TOKEN_DECIMALS},
    state::{
        derive_pool_authority_pda, derive_pool_pda, derive_position_pda, Pool, Position,
        STATE_VERSION,
    },
};

const POOL_ID: u64 = 12;
const MAX_REWARD_RATE_PER_SLOT: u64 = 10_000;
const STAKE_AMOUNT: u64 = 1_000;
const REWARD_FUNDING: u64 = 10_000;

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

fn transfer_tokens(
    ctx: &mut AnchorContext,
    source: Pubkey,
    destination: Pubkey,
    authority: &Keypair,
    amount: u64,
) {
    let ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &source,
        &destination,
        &authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).unwrap();
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

fn read_position(ctx: &AnchorContext, position: Pubkey) -> Position {
    let account = ctx.svm.get_account(&position).unwrap();
    Position::try_deserialize(&mut account.data.as_slice()).unwrap()
}

fn write_position(ctx: &mut AnchorContext, position_pubkey: Pubkey, position: &Position) {
    let mut account = ctx.svm.get_account(&position_pubkey).unwrap();
    account.data.clear();
    position.try_serialize(&mut account.data).unwrap();
    ctx.svm
        .set_account(
            position_pubkey,
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

fn read_token(ctx: &AnchorContext, token_account: Pubkey) -> spl_token::state::Account {
    let account = ctx.svm.get_account(&token_account).unwrap();
    spl_token::state::Account::unpack(&account.data).unwrap()
}

fn write_token_amount(ctx: &mut AnchorContext, token_account_pubkey: Pubkey, amount: u64) {
    let mut account = ctx.svm.get_account(&token_account_pubkey).unwrap();
    let mut token_account = spl_token::state::Account::unpack(&account.data).unwrap();
    token_account.amount = amount;
    spl_token::state::Account::pack(token_account, &mut account.data).unwrap();
    ctx.svm
        .set_account(
            token_account_pubkey,
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

struct Fixture {
    ctx: AnchorContext,
    pool: Pubkey,
    pool_authority: Pubkey,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
    stake_vault: Pubkey,
    reward_vault: Pubkey,
    user: Keypair,
    funder: Keypair,
    admins: [Keypair; ADMIN_COUNT],
    user_stake_ata: Pubkey,
    user_reward_ata: Pubkey,
    funder_reward_account: Pubkey,
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

    let user = Keypair::new();
    let funder = Keypair::new();
    fund_sol(&mut ctx, &user);
    fund_sol(&mut ctx, &funder);

    let user_stake_ata = create_ata(&mut ctx, user.pubkey(), stake_mint);
    let user_reward_ata = create_ata(&mut ctx, user.pubkey(), reward_mint);
    let funder_reward_account = create_token_account(&mut ctx, funder.pubkey(), reward_mint);
    mint_to(&mut ctx, stake_mint, user_stake_ata, STAKE_AMOUNT * 10);
    mint_to(
        &mut ctx,
        reward_mint,
        funder_reward_account,
        REWARD_FUNDING * 10,
    );

    let position = derive_position_pda(&staking_pool::ID, &pool, &user.pubkey()).0;
    let open = anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::OpenPosition {
            user: user.pubkey(),
            pool,
            position,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::OpenPosition {}.data(),
    };
    ctx.execute_instruction(open, &[&user])
        .unwrap()
        .assert_success();

    Fixture {
        ctx,
        pool,
        pool_authority,
        stake_mint,
        reward_mint,
        stake_vault,
        reward_vault,
        user,
        funder,
        admins,
        user_stake_ata,
        user_reward_ata,
        funder_reward_account,
    }
}

fn set_pool_active(fixture: &mut Fixture, reward_rate: u64) {
    let mut pool = read_pool(&fixture.ctx, fixture.pool);
    pool.paused = false;
    pool.reward_rate_per_slot = reward_rate;
    pool.last_update_slot = fixture
        .ctx
        .svm
        .get_sysvar::<solana_sdk::clock::Clock>()
        .slot;
    write_pool(&mut fixture.ctx, fixture.pool, &pool);
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

fn stake_ix(fixture: &Fixture, amount: u64) -> anchor_litesvm::Instruction {
    let position = derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0;
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::Stake {
            user: fixture.user.pubkey(),
            pool: fixture.pool,
            position,
            stake_mint: fixture.stake_mint,
            reward_mint: fixture.reward_mint,
            user_stake_account: fixture.user_stake_ata,
            stake_vault: fixture.stake_vault,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::Stake { amount }.data(),
    }
}

fn unstake_ix(fixture: &Fixture, amount: u64) -> anchor_litesvm::Instruction {
    let position = derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0;
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::Unstake {
            user: fixture.user.pubkey(),
            pool: fixture.pool,
            pool_authority: fixture.pool_authority,
            position,
            stake_mint: fixture.stake_mint,
            reward_mint: fixture.reward_mint,
            user_stake_account: fixture.user_stake_ata,
            stake_vault: fixture.stake_vault,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::Unstake { amount }.data(),
    }
}

fn claim_ix(fixture: &Fixture) -> anchor_litesvm::Instruction {
    let position = derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0;
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::ClaimRewards {
            user: fixture.user.pubkey(),
            pool: fixture.pool,
            pool_authority: fixture.pool_authority,
            position,
            reward_mint: fixture.reward_mint,
            reward_vault: fixture.reward_vault,
            user_reward_account: fixture.user_reward_ata,
            token_program: spl_token::ID,
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

fn emergency_withdraw_ix(fixture: &Fixture) -> anchor_litesvm::Instruction {
    let position = derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0;
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::EmergencyWithdraw {
            user: fixture.user.pubkey(),
            pool: fixture.pool,
            pool_authority: fixture.pool_authority,
            position,
            stake_mint: fixture.stake_mint,
            reward_mint: fixture.reward_mint,
            user_stake_account: fixture.user_stake_ata,
            stake_vault: fixture.stake_vault,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::EmergencyWithdraw {}.data(),
    }
}

fn execute_user(
    fixture: &mut Fixture,
    ix: anchor_litesvm::Instruction,
) -> anchor_litesvm::TransactionResult {
    fixture
        .ctx
        .execute_instruction(ix, &[&fixture.user])
        .unwrap()
}

fn execute_funder(
    fixture: &mut Fixture,
    ix: anchor_litesvm::Instruction,
) -> anchor_litesvm::TransactionResult {
    fixture
        .ctx
        .execute_instruction(ix, &[&fixture.funder])
        .unwrap()
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

fn reward_solvency_holds(fixture: &Fixture) -> bool {
    let pool = read_pool(&fixture.ctx, fixture.pool);
    let vault = read_token(&fixture.ctx, fixture.reward_vault);
    pool.remaining_reward_budget_scaled + pool.allocated_liability_scaled
        <= u128::from(vault.amount) * REWARD_PRECISION
}

#[test]
fn fund_rewards_accepts_valid_source_and_preserves_solvency() {
    let mut fixture = setup();

    {
        let ix = fund_rewards_ix(&fixture, REWARD_FUNDING);
        execute_funder(&mut fixture, ix)
    }
    .assert_success();

    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(
        pool.remaining_reward_budget_scaled,
        u128::from(REWARD_FUNDING) * REWARD_PRECISION
    );
    assert_eq!(
        read_token(&fixture.ctx, fixture.reward_vault).amount,
        REWARD_FUNDING
    );
    assert!(reward_solvency_holds(&fixture));
}

#[test]
fn direct_reward_vault_donation_is_surplus_only() {
    let mut fixture = setup();

    transfer_tokens(
        &mut fixture.ctx,
        fixture.funder_reward_account,
        fixture.reward_vault,
        &fixture.funder,
        REWARD_FUNDING,
    );

    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(pool.remaining_reward_budget_scaled, 0);
    assert_eq!(
        read_token(&fixture.ctx, fixture.reward_vault).amount,
        REWARD_FUNDING
    );
    assert!(reward_solvency_holds(&fixture));
}

#[test]
fn fund_rewards_rejects_zero_and_wrong_authority_without_state_change() {
    let mut fixture = setup();
    let before_pool = read_pool(&fixture.ctx, fixture.pool);
    let before_source = read_token(&fixture.ctx, fixture.funder_reward_account).amount;
    let zero_ix = fund_rewards_ix(&fixture, 0);
    let zero_result = execute_funder(&mut fixture, zero_ix);
    let attacker = Keypair::new();
    fund_sol(&mut fixture.ctx, &attacker);
    let wrong_authority_ix = anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::FundRewards {
            source_authority: attacker.pubkey(),
            pool: fixture.pool,
            source_reward_account: fixture.funder_reward_account,
            reward_mint: fixture.reward_mint,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::FundRewards {
            amount: REWARD_FUNDING,
        }
        .data(),
    };
    let wrong_authority_result = fixture
        .ctx
        .execute_instruction(wrong_authority_ix, &[&attacker])
        .unwrap();

    assert!(!zero_result.is_success());
    assert!(!wrong_authority_result.is_success());
    assert_eq!(read_pool(&fixture.ctx, fixture.pool), before_pool);
    assert_eq!(
        read_token(&fixture.ctx, fixture.funder_reward_account).amount,
        before_source
    );
    assert_eq!(read_token(&fixture.ctx, fixture.reward_vault).amount, 0);
}

#[test]
fn stake_first_deposit_updates_position_pool_and_vault() {
    let mut fixture = setup();
    set_pool_active(&mut fixture, 1);

    {
        let ix = stake_ix(&fixture, STAKE_AMOUNT);
        execute_user(&mut fixture, ix)
    }
    .assert_success();

    let position = read_position(
        &fixture.ctx,
        derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0,
    );
    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(position.version, STATE_VERSION);
    assert_eq!(position.staked_amount, STAKE_AMOUNT);
    assert_eq!(position.reward_debt_scaled, 0);
    assert_eq!(pool.total_staked, STAKE_AMOUNT);
    assert_eq!(
        read_token(&fixture.ctx, fixture.stake_vault).amount,
        STAKE_AMOUNT
    );
    assert!(read_token(&fixture.ctx, fixture.stake_vault).amount >= pool.total_staked);
}

#[test]
fn stake_rejects_paused_zero_and_insufficient_balance_with_rollback() {
    let mut fixture = setup();
    let before_pool = read_pool(&fixture.ctx, fixture.pool);
    let before_user_balance = read_token(&fixture.ctx, fixture.user_stake_ata).amount;

    let paused_result = {
        let ix = stake_ix(&fixture, STAKE_AMOUNT);
        execute_user(&mut fixture, ix)
    };
    set_pool_active(&mut fixture, 1);
    let zero_result = {
        let ix = stake_ix(&fixture, 0);
        execute_user(&mut fixture, ix)
    };
    let excessive_result = {
        let ix = stake_ix(&fixture, before_user_balance + 1);
        execute_user(&mut fixture, ix)
    };

    assert!(!paused_result.is_success());
    assert!(!zero_result.is_success());
    assert!(!excessive_result.is_success());
    assert_eq!(
        read_pool(&fixture.ctx, fixture.pool).total_staked,
        before_pool.total_staked
    );
    assert_eq!(
        read_token(&fixture.ctx, fixture.user_stake_ata).amount,
        before_user_balance
    );
    assert_eq!(read_token(&fixture.ctx, fixture.stake_vault).amount, 0);
}

#[test]
fn stake_rejects_wrong_user_ata() {
    let mut fixture = setup();
    set_pool_active(&mut fixture, 1);
    let wrong_ata =
        create_token_account(&mut fixture.ctx, fixture.user.pubkey(), fixture.stake_mint);
    let position = derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0;
    let ix = anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::Stake {
            user: fixture.user.pubkey(),
            pool: fixture.pool,
            position,
            stake_mint: fixture.stake_mint,
            reward_mint: fixture.reward_mint,
            user_stake_account: wrong_ata,
            stake_vault: fixture.stake_vault,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::Stake {
            amount: STAKE_AMOUNT,
        }
        .data(),
    };

    let result = execute_user(&mut fixture, ix);

    assert!(!result.is_success());
    assert_eq!(read_token(&fixture.ctx, fixture.stake_vault).amount, 0);
}

#[test]
fn unstake_partial_returns_principal_and_preserves_rewards_even_when_paused() {
    let mut fixture = setup();
    {
        let ix = fund_rewards_ix(&fixture, REWARD_FUNDING);
        execute_funder(&mut fixture, ix)
    }
    .assert_success();
    set_pool_active(&mut fixture, 10);
    {
        let ix = stake_ix(&fixture, STAKE_AMOUNT);
        execute_user(&mut fixture, ix)
    }
    .assert_success();
    let start_slot = read_pool(&fixture.ctx, fixture.pool).last_update_slot;
    fixture.ctx.svm.warp_to_slot(start_slot + 5);
    let mut pool = read_pool(&fixture.ctx, fixture.pool);
    pool.paused = true;
    write_pool(&mut fixture.ctx, fixture.pool, &pool);

    {
        let ix = unstake_ix(&fixture, STAKE_AMOUNT / 2);
        execute_user(&mut fixture, ix)
    }
    .assert_success();

    let position = read_position(
        &fixture.ctx,
        derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0,
    );
    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(position.staked_amount, STAKE_AMOUNT / 2);
    assert_eq!(pool.total_staked, STAKE_AMOUNT / 2);
    assert_eq!(
        read_token(&fixture.ctx, fixture.stake_vault).amount,
        STAKE_AMOUNT / 2
    );
    assert_eq!(
        read_token(&fixture.ctx, fixture.user_stake_ata).amount,
        STAKE_AMOUNT * 10 - STAKE_AMOUNT / 2
    );
}

#[test]
fn unstake_rejects_excessive_amount_and_wrong_pool_authority() {
    let mut fixture = setup();
    set_pool_active(&mut fixture, 1);
    {
        let ix = stake_ix(&fixture, STAKE_AMOUNT);
        execute_user(&mut fixture, ix)
    }
    .assert_success();
    let excessive = {
        let ix = unstake_ix(&fixture, STAKE_AMOUNT + 1);
        execute_user(&mut fixture, ix)
    };
    let position = derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0;
    let bad_authority = Pubkey::new_unique();
    let wrong_authority_ix = anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::Unstake {
            user: fixture.user.pubkey(),
            pool: fixture.pool,
            pool_authority: bad_authority,
            position,
            stake_mint: fixture.stake_mint,
            reward_mint: fixture.reward_mint,
            user_stake_account: fixture.user_stake_ata,
            stake_vault: fixture.stake_vault,
            reward_vault: fixture.reward_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::Unstake {
            amount: STAKE_AMOUNT / 2,
        }
        .data(),
    };
    let wrong_authority = execute_user(&mut fixture, wrong_authority_ix);

    assert!(!excessive.is_success());
    assert!(!wrong_authority.is_success());
    assert_eq!(
        read_token(&fixture.ctx, fixture.stake_vault).amount,
        STAKE_AMOUNT
    );
}

#[test]
fn claim_transfers_whole_rewards_and_prevents_double_payment() {
    let mut fixture = setup();
    {
        let ix = fund_rewards_ix(&fixture, REWARD_FUNDING);
        execute_funder(&mut fixture, ix)
    }
    .assert_success();
    set_pool_active(&mut fixture, 10);
    {
        let ix = stake_ix(&fixture, STAKE_AMOUNT);
        execute_user(&mut fixture, ix)
    }
    .assert_success();
    let start_slot = read_pool(&fixture.ctx, fixture.pool).last_update_slot;
    fixture.ctx.svm.warp_to_slot(start_slot + 5);

    {
        let ix = claim_ix(&fixture);
        execute_user(&mut fixture, ix)
    }
    .assert_success();
    let first_reward_balance = read_token(&fixture.ctx, fixture.user_reward_ata).amount;
    let second_result = {
        let ix = claim_ix(&fixture);
        execute_user(&mut fixture, ix)
    };

    assert_eq!(first_reward_balance, 50);
    assert!(!second_result.is_success());
    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(pool.allocated_liability_scaled, 0);
    assert!(reward_solvency_holds(&fixture));
}

#[test]
fn claim_rejects_while_paused_wrong_ata_and_insufficient_backing() {
    let mut fixture = setup();
    {
        let ix = fund_rewards_ix(&fixture, REWARD_FUNDING);
        execute_funder(&mut fixture, ix)
    }
    .assert_success();
    set_pool_active(&mut fixture, 10);
    {
        let ix = stake_ix(&fixture, STAKE_AMOUNT);
        execute_user(&mut fixture, ix)
    }
    .assert_success();
    let start_slot = read_pool(&fixture.ctx, fixture.pool).last_update_slot;
    fixture.ctx.svm.warp_to_slot(start_slot + 5);

    let mut pool = read_pool(&fixture.ctx, fixture.pool);
    pool.paused = true;
    write_pool(&mut fixture.ctx, fixture.pool, &pool);
    let paused_claim_result = {
        let ix = claim_ix(&fixture);
        execute_user(&mut fixture, ix)
    };
    assert!(!paused_claim_result.is_success());
    pool = read_pool(&fixture.ctx, fixture.pool);
    pool.paused = false;
    write_pool(&mut fixture.ctx, fixture.pool, &pool);

    let wrong_reward_ata =
        create_token_account(&mut fixture.ctx, fixture.user.pubkey(), fixture.reward_mint);
    let position = derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0;
    let wrong_ata_ix = anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::ClaimRewards {
            user: fixture.user.pubkey(),
            pool: fixture.pool,
            pool_authority: fixture.pool_authority,
            position,
            reward_mint: fixture.reward_mint,
            reward_vault: fixture.reward_vault,
            user_reward_account: wrong_reward_ata,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::ClaimRewards {}.data(),
    };
    assert!(!execute_user(&mut fixture, wrong_ata_ix).is_success());

    write_token_amount(&mut fixture.ctx, fixture.reward_vault, 0);
    let insufficient_backing_result = {
        let ix = claim_ix(&fixture);
        execute_user(&mut fixture, ix)
    };
    assert!(!insufficient_backing_result.is_success());
    assert_eq!(read_token(&fixture.ctx, fixture.user_reward_ata).amount, 0);
}

#[test]
fn pause_by_admin_checkpoints_and_blocks_reward_generation() {
    let mut fixture = setup();
    {
        let ix = fund_rewards_ix(&fixture, REWARD_FUNDING);
        execute_funder(&mut fixture, ix)
    }
    .assert_success();
    set_pool_active(&mut fixture, 10);
    {
        let ix = stake_ix(&fixture, STAKE_AMOUNT);
        execute_user(&mut fixture, ix)
    }
    .assert_success();

    let start_slot = read_pool(&fixture.ctx, fixture.pool).last_update_slot;
    fixture.ctx.svm.warp_to_slot(start_slot + 5);
    {
        let ix = pause_ix(&fixture, fixture.admins[0].pubkey());
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();

    let paused_pool = read_pool(&fixture.ctx, fixture.pool);
    let paused_budget = paused_pool.remaining_reward_budget_scaled;
    let paused_liability = paused_pool.allocated_liability_scaled;
    assert!(paused_pool.paused);
    assert_eq!(paused_liability, 50 * REWARD_PRECISION);

    fixture
        .ctx
        .svm
        .warp_to_slot(paused_pool.last_update_slot + 20);
    {
        let ix = unstake_ix(&fixture, STAKE_AMOUNT / 2);
        execute_user(&mut fixture, ix)
    }
    .assert_success();

    let after_unstake = read_pool(&fixture.ctx, fixture.pool);
    assert!(after_unstake.paused);
    assert_eq!(after_unstake.remaining_reward_budget_scaled, paused_budget);
    assert_eq!(after_unstake.allocated_liability_scaled, paused_liability);
}

#[test]
fn pause_rejects_non_admin_and_redundant_pause_without_state_change() {
    let mut fixture = setup();
    set_pool_active(&mut fixture, 1);
    let before = read_pool(&fixture.ctx, fixture.pool);
    let attacker = Keypair::new();
    fund_sol(&mut fixture.ctx, &attacker);
    let unauthorized_ix = pause_ix(&fixture, attacker.pubkey());
    let unauthorized = fixture
        .ctx
        .execute_instruction(unauthorized_ix, &[&attacker])
        .unwrap();
    assert!(!unauthorized.is_success());
    assert_eq!(read_pool(&fixture.ctx, fixture.pool), before);

    {
        let ix = pause_ix(&fixture, fixture.admins[0].pubkey());
        execute_admin(&mut fixture, ix, 0)
    }
    .assert_success();
    let redundant = {
        let ix = pause_ix(&fixture, fixture.admins[1].pubkey());
        execute_admin(&mut fixture, ix, 1)
    };
    assert!(!redundant.is_success());
    assert!(read_pool(&fixture.ctx, fixture.pool).paused);
}

#[test]
fn emergency_withdraw_returns_principal_and_recycles_pending_rewards() {
    let mut fixture = setup();
    {
        let ix = fund_rewards_ix(&fixture, REWARD_FUNDING);
        execute_funder(&mut fixture, ix)
    }
    .assert_success();
    set_pool_active(&mut fixture, 10);
    {
        let ix = stake_ix(&fixture, STAKE_AMOUNT);
        execute_user(&mut fixture, ix)
    }
    .assert_success();
    let start_slot = read_pool(&fixture.ctx, fixture.pool).last_update_slot;
    fixture.ctx.svm.warp_to_slot(start_slot + 5);

    {
        let ix = emergency_withdraw_ix(&fixture);
        execute_user(&mut fixture, ix)
    }
    .assert_success();

    let position = read_position(
        &fixture.ctx,
        derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0,
    );
    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(position.staked_amount, 0);
    assert_eq!(position.reward_debt_scaled, 0);
    assert_eq!(position.pending_reward_scaled, 0);
    assert_eq!(pool.total_staked, 0);
    assert_eq!(pool.allocated_liability_scaled, 0);
    assert_eq!(
        pool.remaining_reward_budget_scaled,
        u128::from(REWARD_FUNDING) * REWARD_PRECISION
    );
    assert_eq!(
        read_token(&fixture.ctx, fixture.user_stake_ata).amount,
        STAKE_AMOUNT * 10
    );
    assert_eq!(read_token(&fixture.ctx, fixture.stake_vault).amount, 0);
    assert!(reward_solvency_holds(&fixture));
}

#[test]
fn emergency_withdraw_allows_fraction_only_forfeiture_while_paused() {
    let mut fixture = setup();
    let position_pubkey =
        derive_position_pda(&staking_pool::ID, &fixture.pool, &fixture.user.pubkey()).0;
    let mut pool = read_pool(&fixture.ctx, fixture.pool);
    pool.paused = true;
    pool.remaining_reward_budget_scaled = 1_000;
    pool.allocated_liability_scaled = 7;
    write_pool(&mut fixture.ctx, fixture.pool, &pool);
    let mut position = read_position(&fixture.ctx, position_pubkey);
    position.pending_reward_scaled = 7;
    write_position(&mut fixture.ctx, position_pubkey, &position);

    {
        let ix = emergency_withdraw_ix(&fixture);
        execute_user(&mut fixture, ix)
    }
    .assert_success();

    let position = read_position(&fixture.ctx, position_pubkey);
    let pool = read_pool(&fixture.ctx, fixture.pool);
    assert_eq!(position.pending_reward_scaled, 0);
    assert_eq!(pool.allocated_liability_scaled, 0);
    assert_eq!(pool.remaining_reward_budget_scaled, 1_007);
}
