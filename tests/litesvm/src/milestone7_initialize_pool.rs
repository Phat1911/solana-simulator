//! Milestone 7: executable LiteSVM coverage for `initialize_pool`.

#![allow(clippy::unwrap_used)]

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, Keypair, Pubkey, Signer};
use solana_program_pack::Pack;
use solana_sdk::transaction::Transaction;
use staking_pool::{
    constants::{ADMIN_COUNT, DEVNET_MAX_REWARD_RATE_PER_SLOT, TOKEN_DECIMALS},
    state::{derive_pool_authority_pda, derive_pool_pda, Pool, POOL_AUTHORITY_SEED},
};

const POOL_ID: u64 = 7;
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

fn create_mint(ctx: &mut AnchorContext, decimals: u8) -> Keypair {
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
        decimals,
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

fn initialize_pool_instruction(
    initializer: Pubkey,
    pool_id: u64,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
    admins: [Pubkey; ADMIN_COUNT],
    max_reward_rate_per_slot: u64,
) -> anchor_litesvm::Instruction {
    let (pool, _) = derive_pool_pda(&staking_pool::ID, &initializer, pool_id);
    let (pool_authority, _) = derive_pool_authority_pda(&staking_pool::ID, &pool);
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

    anchor_litesvm::Instruction {
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
            admins,
            max_reward_rate_per_slot,
        }
        .data(),
    }
}

fn read_pool(ctx: &AnchorContext, pool: Pubkey) -> Pool {
    let account = ctx.svm.get_account(&pool).unwrap();
    Pool::try_deserialize(&mut account.data.as_slice()).unwrap()
}

fn read_token_account(pubkey: Pubkey, ctx: &AnchorContext) -> spl_token::state::Account {
    let account = ctx.svm.get_account(&pubkey).unwrap();
    spl_token::state::Account::unpack(&account.data).unwrap()
}

fn execute_initialize(
    ctx: &mut AnchorContext,
    pool_id: u64,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
    admins: [Pubkey; ADMIN_COUNT],
    max_reward_rate_per_slot: u64,
) -> anchor_litesvm::TransactionResult {
    let initializer = ctx.payer().pubkey();
    let ix = initialize_pool_instruction(
        initializer,
        pool_id,
        stake_mint,
        reward_mint,
        admins,
        max_reward_rate_per_slot,
    );
    let payer = ctx.payer().insecure_clone();
    ctx.execute_instruction(ix, &[&payer]).unwrap()
}

#[test]
fn initialize_pool_creates_paused_pool_and_pda_owned_vaults() {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();
    let reward_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();
    let admins = default_admins();
    let initializer = ctx.payer().pubkey();
    let (pool, pool_bump) = derive_pool_pda(&staking_pool::ID, &initializer, POOL_ID);
    let (pool_authority, pool_authority_bump) = derive_pool_authority_pda(&staking_pool::ID, &pool);

    let result = execute_initialize(
        &mut ctx,
        POOL_ID,
        stake_mint,
        reward_mint,
        admins,
        MAX_REWARD_RATE_PER_SLOT,
    );
    result.assert_success();

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
    let pool_state = read_pool(&ctx, pool);
    let stake_vault_state = read_token_account(stake_vault, &ctx);
    let reward_vault_state = read_token_account(reward_vault, &ctx);

    assert_eq!(pool_state.initializer, initializer);
    assert_eq!(pool_state.pool_id, POOL_ID);
    assert_eq!(pool_state.pool_bump, pool_bump);
    assert_eq!(pool_state.pool_authority_bump, pool_authority_bump);
    assert_eq!(pool_state.stake_mint, stake_mint);
    assert_eq!(pool_state.reward_mint, reward_mint);
    assert_eq!(pool_state.stake_vault, stake_vault);
    assert_eq!(pool_state.reward_vault, reward_vault);
    assert_eq!(pool_state.admins, admins);
    assert!(pool_state.paused);
    assert_eq!(pool_state.reward_rate_per_slot, 0);
    assert_eq!(
        pool_state.max_reward_rate_per_slot,
        MAX_REWARD_RATE_PER_SLOT
    );
    assert_eq!(pool_state.total_staked, 0);
    assert_eq!(pool_state.acc_reward_per_stake_scaled, 0);
    assert_eq!(pool_state.remaining_reward_budget_scaled, 0);
    assert_eq!(pool_state.allocated_liability_scaled, 0);
    assert_eq!(stake_vault_state.owner, pool_authority);
    assert_eq!(stake_vault_state.mint, stake_mint);
    assert_eq!(reward_vault_state.owner, pool_authority);
    assert_eq!(reward_vault_state.mint, reward_mint);
}

#[test]
fn initialize_pool_rejects_duplicate_admins() {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();
    let reward_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();
    let duplicate_admin = Pubkey::new_unique();
    let admins = [duplicate_admin, duplicate_admin, Pubkey::new_unique()];

    let result = execute_initialize(
        &mut ctx,
        POOL_ID,
        stake_mint,
        reward_mint,
        admins,
        MAX_REWARD_RATE_PER_SLOT,
    );

    assert!(!result.is_success());
}

#[test]
fn initialize_pool_rejects_wrong_mint_decimals() {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx, TOKEN_DECIMALS + 1).pubkey();
    let reward_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();

    let result = execute_initialize(
        &mut ctx,
        POOL_ID,
        stake_mint,
        reward_mint,
        default_admins(),
        MAX_REWARD_RATE_PER_SLOT,
    );

    assert!(!result.is_success());
}

#[test]
fn initialize_pool_rejects_same_stake_and_reward_mint() {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();

    let result = execute_initialize(
        &mut ctx,
        POOL_ID,
        stake_mint,
        stake_mint,
        default_admins(),
        MAX_REWARD_RATE_PER_SLOT,
    );

    assert!(!result.is_success());
}

#[test]
fn initialize_pool_rejects_rate_above_devnet_maximum() {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();
    let reward_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();

    let result = execute_initialize(
        &mut ctx,
        POOL_ID,
        stake_mint,
        reward_mint,
        default_admins(),
        DEVNET_MAX_REWARD_RATE_PER_SLOT + 1,
    );

    assert!(!result.is_success());
}

#[test]
fn initialize_pool_rejects_duplicate_initialization() {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();
    let reward_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();
    let admins = default_admins();

    execute_initialize(
        &mut ctx,
        POOL_ID,
        stake_mint,
        reward_mint,
        admins,
        MAX_REWARD_RATE_PER_SLOT,
    )
    .assert_success();
    let second_result = execute_initialize(
        &mut ctx,
        POOL_ID,
        stake_mint,
        reward_mint,
        admins,
        MAX_REWARD_RATE_PER_SLOT,
    );

    assert!(!second_result.is_success());
}

#[test]
fn initialize_pool_rejects_non_canonical_pool_authority() {
    let mut ctx = new_context();
    let stake_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();
    let reward_mint = create_mint(&mut ctx, TOKEN_DECIMALS).pubkey();
    let initializer = ctx.payer().pubkey();
    let (pool, _) = derive_pool_pda(&staking_pool::ID, &initializer, POOL_ID);
    let wrong_pool_authority = Pubkey::find_program_address(
        &[POOL_AUTHORITY_SEED, initializer.as_ref()],
        &staking_pool::ID,
    )
    .0;
    let stake_vault = spl_associated_token_account::get_associated_token_address_with_program_id(
        &wrong_pool_authority,
        &stake_mint,
        &spl_token::ID,
    );
    let reward_vault = spl_associated_token_account::get_associated_token_address_with_program_id(
        &wrong_pool_authority,
        &reward_mint,
        &spl_token::ID,
    );

    let ix = anchor_litesvm::Instruction {
        program_id: staking_pool::ID,
        accounts: staking_pool::accounts::InitializePool {
            initializer,
            pool,
            pool_authority: wrong_pool_authority,
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
            admins: default_admins(),
            max_reward_rate_per_slot: MAX_REWARD_RATE_PER_SLOT,
        }
        .data(),
    };
    let payer = ctx.payer().insecure_clone();
    let result = ctx.execute_instruction(ix, &[&payer]).unwrap();

    assert!(!result.is_success());
}
