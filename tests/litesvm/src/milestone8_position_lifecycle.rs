//! Milestone 8: executable LiteSVM coverage for position open/close lifecycle.

#![allow(clippy::unwrap_used)]

use anchor_lang::{AccountDeserialize, AccountSerialize, InstructionData, ToAccountMetas};
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, Keypair, Pubkey, Signer};
use solana_program_pack::Pack;
use solana_sdk::{account::Account, transaction::Transaction};
use staking_pool::{
    constants::{ADMIN_COUNT, TOKEN_DECIMALS},
    state::{derive_pool_pda, derive_position_pda, Position, POSITION_SEED, STATE_VERSION},
};

const POOL_ID: u64 = 8;
const SECOND_POOL_ID: u64 = 9;
const MAX_REWARD_RATE_PER_SLOT: u64 = 10_000;

fn program_bytes() -> &'static [u8] {
    include_bytes!("../../../target/deploy/staking_pool.so")
}

fn new_context() -> AnchorContext {
    AnchorLiteSVM::build_with_program(staking_pool::ID, program_bytes())
}

fn default_admins() -> [Pubkey; ADMIN_COUNT] {
    [
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
    ]
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

fn initialize_pool(
    ctx: &mut AnchorContext,
    pool_id: u64,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
) -> Pubkey {
    let initializer = ctx.payer().pubkey();
    let (pool, _) = derive_pool_pda(&staking_pool::ID, &initializer, pool_id);
    let (pool_authority, _) =
        staking_pool::state::derive_pool_authority_pda(&staking_pool::ID, &pool);
    let stake_vault = spl_associated_token_account::get_associated_token_address_with_program_id(
        &pool_authority,
        &stake_mint,
        &spl_token::ID,
    );
    let reward_vault = spl_associated_token_account::get_associated_token_address_with_program_id(
        &pool_authority,
        &reward_mint,
        &spl_token::ID,
    );

    let ix = anchor_litesvm::Instruction {
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
            pool_id,
            admins: default_admins(),
            max_reward_rate_per_slot: MAX_REWARD_RATE_PER_SLOT,
        }
        .data(),
    };

    let payer = ctx.payer().insecure_clone();
    ctx.execute_instruction(ix, &[&payer])
        .unwrap()
        .assert_success();
    pool
}

fn open_position_instruction(
    user: Pubkey,
    pool: Pubkey,
    position: Pubkey,
) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::OpenPosition {
            user,
            pool,
            position,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::OpenPosition {}.data(),
    }
}

fn close_position_instruction(
    user: Pubkey,
    pool: Pubkey,
    position: Pubkey,
) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::ClosePosition {
            user,
            pool,
            position,
        }
        .to_account_metas(None),
        data: staking_pool::instruction::ClosePosition {}.data(),
    }
}

fn execute_open(
    ctx: &mut AnchorContext,
    user: &Keypair,
    pool: Pubkey,
) -> anchor_litesvm::TransactionResult {
    let (position, _) = derive_position_pda(&staking_pool::ID, &pool, &user.pubkey());
    let ix = open_position_instruction(user.pubkey(), pool, position);
    ctx.execute_instruction(ix, &[user]).unwrap()
}

fn execute_close(
    ctx: &mut AnchorContext,
    user: &Keypair,
    pool: Pubkey,
) -> anchor_litesvm::TransactionResult {
    let (position, _) = derive_position_pda(&staking_pool::ID, &pool, &user.pubkey());
    let ix = close_position_instruction(user.pubkey(), pool, position);
    ctx.execute_instruction(ix, &[user]).unwrap()
}

fn fund_user(ctx: &mut AnchorContext, user: &Keypair) {
    let payer = ctx.payer();
    let transfer = solana_system_interface::instruction::transfer(
        &payer.pubkey(),
        &user.pubkey(),
        1_000_000_000,
    );
    let tx = Transaction::new_signed_with_payer(
        &[transfer],
        Some(&payer.pubkey()),
        &[payer],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).unwrap();
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

fn setup_pool_and_user() -> (AnchorContext, Pubkey, Keypair) {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx).pubkey();
    let reward_mint = create_mint(&mut ctx).pubkey();
    let pool = initialize_pool(&mut ctx, POOL_ID, stake_mint, reward_mint);
    let user = Keypair::new();
    fund_user(&mut ctx, &user);
    (ctx, pool, user)
}

#[test]
fn open_position_creates_empty_canonical_position() {
    let (mut ctx, pool, user) = setup_pool_and_user();
    let (position, bump) = derive_position_pda(&staking_pool::ID, &pool, &user.pubkey());

    execute_open(&mut ctx, &user, pool).assert_success();

    let position_state = read_position(&ctx, position);
    assert_eq!(position_state.version, STATE_VERSION);
    assert_eq!(position_state.pool, pool);
    assert_eq!(position_state.owner, user.pubkey());
    assert_eq!(position_state.bump, bump);
    assert_eq!(position_state.staked_amount, 0);
    assert_eq!(position_state.reward_debt_scaled, 0);
    assert_eq!(position_state.pending_reward_scaled, 0);
}

#[test]
fn open_position_rejects_duplicate_position() {
    let (mut ctx, pool, user) = setup_pool_and_user();

    execute_open(&mut ctx, &user, pool).assert_success();
    let result = execute_open(&mut ctx, &user, pool);

    assert!(!result.is_success());
}

#[test]
fn open_position_rejects_wrong_position_seed() {
    let (mut ctx, pool, user) = setup_pool_and_user();
    let wrong_position = Pubkey::find_program_address(
        &[POSITION_SEED, pool.as_ref(), Pubkey::new_unique().as_ref()],
        &staking_pool::ID,
    )
    .0;

    let ix = open_position_instruction(user.pubkey(), pool, wrong_position);
    let result = ctx.execute_instruction(ix, &[&user]).unwrap();

    assert!(!result.is_success());
    assert!(ctx.svm.get_account(&wrong_position).is_none());
}

#[test]
fn open_position_rejects_wrong_pool_seed() {
    let mut ctx = new_context();
    let first_stake_mint = create_mint(&mut ctx).pubkey();
    let first_reward_mint = create_mint(&mut ctx).pubkey();
    let second_stake_mint = create_mint(&mut ctx).pubkey();
    let second_reward_mint = create_mint(&mut ctx).pubkey();
    let first_pool = initialize_pool(&mut ctx, POOL_ID, first_stake_mint, first_reward_mint);
    let second_pool = initialize_pool(
        &mut ctx,
        SECOND_POOL_ID,
        second_stake_mint,
        second_reward_mint,
    );
    let user = Keypair::new();
    fund_user(&mut ctx, &user);
    let wrong_position = derive_position_pda(&staking_pool::ID, &first_pool, &user.pubkey()).0;

    let ix = open_position_instruction(user.pubkey(), second_pool, wrong_position);
    let result = ctx.execute_instruction(ix, &[&user]).unwrap();

    assert!(!result.is_success());
    assert!(ctx.svm.get_account(&wrong_position).is_none());
}

#[test]
fn open_position_separates_users_and_pools() {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx).pubkey();
    let reward_mint = create_mint(&mut ctx).pubkey();
    let first_pool = initialize_pool(&mut ctx, POOL_ID, stake_mint, reward_mint);
    let second_pool = initialize_pool(&mut ctx, SECOND_POOL_ID, stake_mint, reward_mint);
    let first_user = Keypair::new();
    let second_user = Keypair::new();
    fund_user(&mut ctx, &first_user);
    fund_user(&mut ctx, &second_user);

    execute_open(&mut ctx, &first_user, first_pool).assert_success();
    execute_open(&mut ctx, &second_user, first_pool).assert_success();
    execute_open(&mut ctx, &first_user, second_pool).assert_success();

    let first_position =
        derive_position_pda(&staking_pool::ID, &first_pool, &first_user.pubkey()).0;
    let second_user_position =
        derive_position_pda(&staking_pool::ID, &first_pool, &second_user.pubkey()).0;
    let second_pool_position =
        derive_position_pda(&staking_pool::ID, &second_pool, &first_user.pubkey()).0;

    assert_ne!(first_position, second_user_position);
    assert_ne!(first_position, second_pool_position);
    assert!(ctx.svm.get_account(&first_position).is_some());
    assert!(ctx.svm.get_account(&second_user_position).is_some());
    assert!(ctx.svm.get_account(&second_pool_position).is_some());
}

#[test]
fn close_position_rejects_unauthorized_signer() {
    let (mut ctx, pool, user) = setup_pool_and_user();
    let attacker = Keypair::new();
    fund_user(&mut ctx, &attacker);
    let (user_position, _) = derive_position_pda(&staking_pool::ID, &pool, &user.pubkey());

    execute_open(&mut ctx, &user, pool).assert_success();
    let ix = close_position_instruction(attacker.pubkey(), pool, user_position);
    let result = ctx.execute_instruction(ix, &[&attacker]).unwrap();

    assert!(!result.is_success());
    assert!(ctx.svm.get_account(&user_position).is_some());
}

#[test]
fn close_position_rejects_non_empty_stake() {
    let (mut ctx, pool, user) = setup_pool_and_user();
    let position_pubkey = derive_position_pda(&staking_pool::ID, &pool, &user.pubkey()).0;
    execute_open(&mut ctx, &user, pool).assert_success();
    let mut position = read_position(&ctx, position_pubkey);
    position.staked_amount = 1;
    write_position(&mut ctx, position_pubkey, &position);

    let result = execute_close(&mut ctx, &user, pool);

    assert!(!result.is_success());
    assert!(ctx.svm.get_account(&position_pubkey).is_some());
}

#[test]
fn close_position_rejects_pending_reward() {
    let (mut ctx, pool, user) = setup_pool_and_user();
    let position_pubkey = derive_position_pda(&staking_pool::ID, &pool, &user.pubkey()).0;
    execute_open(&mut ctx, &user, pool).assert_success();
    let mut position = read_position(&ctx, position_pubkey);
    position.pending_reward_scaled = 1;
    write_position(&mut ctx, position_pubkey, &position);

    let result = execute_close(&mut ctx, &user, pool);

    assert!(!result.is_success());
    assert!(ctx.svm.get_account(&position_pubkey).is_some());
}

#[test]
fn close_position_rejects_reward_debt() {
    let (mut ctx, pool, user) = setup_pool_and_user();
    let position_pubkey = derive_position_pda(&staking_pool::ID, &pool, &user.pubkey()).0;
    execute_open(&mut ctx, &user, pool).assert_success();
    let mut position = read_position(&ctx, position_pubkey);
    position.reward_debt_scaled = 1;
    write_position(&mut ctx, position_pubkey, &position);

    let result = execute_close(&mut ctx, &user, pool);

    assert!(!result.is_success());
    assert!(ctx.svm.get_account(&position_pubkey).is_some());
}

#[test]
fn close_position_closes_empty_position_and_returns_rent() {
    let (mut ctx, pool, user) = setup_pool_and_user();
    let position_pubkey = derive_position_pda(&staking_pool::ID, &pool, &user.pubkey()).0;
    execute_open(&mut ctx, &user, pool).assert_success();
    let user_lamports_before_close = ctx.svm.get_account(&user.pubkey()).unwrap().lamports;
    let position_rent = ctx.svm.get_account(&position_pubkey).unwrap().lamports;

    execute_close(&mut ctx, &user, pool).assert_success();

    let user_lamports_after_close = ctx.svm.get_account(&user.pubkey()).unwrap().lamports;
    let closed_position = ctx.svm.get_account(&position_pubkey).unwrap();
    assert_eq!(closed_position.lamports, 0);
    assert!(closed_position.data.iter().all(|byte| *byte == 0));
    assert!(position_rent > 0);
    assert!(user_lamports_after_close > user_lamports_before_close);
}
