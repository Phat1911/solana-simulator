//! Milestone 17: LiteSVM coverage for the Devnet-only faucet program.

#![allow(clippy::unwrap_used)]

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, Keypair, Pubkey, Signer};
use solana_program_pack::Pack;
use solana_sdk::transaction::Transaction;

use demo_faucet::{
    derive_faucet_authority_pda, derive_faucet_claim_pda, FaucetClaimReceipt, FAUCET_CLAIM_AMOUNT,
    STATE_VERSION, TOKEN_DECIMALS,
};

fn program_bytes() -> &'static [u8] {
    include_bytes!("../../../target/deploy/demo_faucet.so")
}

fn new_context() -> AnchorContext {
    AnchorLiteSVM::build_with_program(demo_faucet::ID, program_bytes())
}

fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address_with_program_id(
        owner,
        mint,
        &spl_token::ID,
    )
}

fn create_mint(ctx: &mut AnchorContext, mint: &Keypair, authority: Pubkey, decimals: u8) -> Pubkey {
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
        &authority,
        None,
        decimals,
    )
    .unwrap();
    let payer = ctx.payer();
    let tx = Transaction::new_signed_with_payer(
        &[create_account, initialize_mint],
        Some(&payer_pubkey),
        &[payer, mint],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).unwrap();
    mint.pubkey()
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

fn read_receipt(ctx: &AnchorContext, receipt: Pubkey) -> FaucetClaimReceipt {
    let account = ctx.svm.get_account(&receipt).unwrap();
    FaucetClaimReceipt::try_deserialize(&mut account.data.as_slice()).unwrap()
}

fn read_token(ctx: &AnchorContext, token_account: Pubkey) -> spl_token::state::Account {
    let account = ctx.svm.get_account(&token_account).unwrap();
    spl_token::state::Account::unpack(&account.data).unwrap()
}

struct Fixture {
    ctx: AnchorContext,
    claimant: Keypair,
    stake_mint: Pubkey,
    faucet_authority: Pubkey,
    claimant_stake_ata: Pubkey,
    claim_receipt: Pubkey,
}

fn setup() -> Fixture {
    let mut ctx = new_context();
    let claimant = Keypair::new();
    fund_sol(&mut ctx, &claimant);
    let stake_mint_keypair = Keypair::new();
    let (faucet_authority, _) =
        derive_faucet_authority_pda(&demo_faucet::ID, &stake_mint_keypair.pubkey());
    let stake_mint = create_mint(
        &mut ctx,
        &stake_mint_keypair,
        faucet_authority,
        TOKEN_DECIMALS,
    );
    let claimant_stake_ata = ata(&claimant.pubkey(), &stake_mint);
    let claim_receipt =
        derive_faucet_claim_pda(&demo_faucet::ID, &stake_mint, &claimant.pubkey()).0;

    Fixture {
        ctx,
        claimant,
        stake_mint,
        faucet_authority,
        claimant_stake_ata,
        claim_receipt,
    }
}

fn claim_ix(fixture: &Fixture) -> anchor_litesvm::Instruction {
    anchor_litesvm::Instruction {
        program_id: demo_faucet::ID,
        accounts: demo_faucet::accounts::ClaimTestStake {
            claimant: fixture.claimant.pubkey(),
            stake_mint: fixture.stake_mint,
            faucet_authority: fixture.faucet_authority,
            claimant_stake_account: fixture.claimant_stake_ata,
            claim_receipt: fixture.claim_receipt,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: demo_faucet::instruction::ClaimTestStake {}.data(),
    }
}

fn execute_claim(
    fixture: &mut Fixture,
    ix: anchor_litesvm::Instruction,
) -> anchor_litesvm::TransactionResult {
    fixture
        .ctx
        .execute_instruction(ix, &[&fixture.claimant])
        .unwrap()
}

#[test]
fn claim_test_stake_mints_once_to_canonical_ata_and_records_receipt() {
    let mut fixture = setup();

    {
        let ix = claim_ix(&fixture);
        execute_claim(&mut fixture, ix)
    }
    .assert_success();

    let receipt = read_receipt(&fixture.ctx, fixture.claim_receipt);
    assert_eq!(receipt.version, STATE_VERSION);
    assert_eq!(receipt.stake_mint, fixture.stake_mint);
    assert_eq!(receipt.claimant, fixture.claimant.pubkey());
    assert_eq!(receipt.amount, FAUCET_CLAIM_AMOUNT);
    assert_eq!(
        read_token(&fixture.ctx, fixture.claimant_stake_ata).amount,
        FAUCET_CLAIM_AMOUNT
    );
}

#[test]
fn claim_test_stake_rejects_replay_without_extra_mint() {
    let mut fixture = setup();
    {
        let ix = claim_ix(&fixture);
        execute_claim(&mut fixture, ix)
    }
    .assert_success();

    let replay = {
        let ix = claim_ix(&fixture);
        execute_claim(&mut fixture, ix)
    };

    assert!(!replay.is_success());
    assert_eq!(
        read_token(&fixture.ctx, fixture.claimant_stake_ata).amount,
        FAUCET_CLAIM_AMOUNT
    );
}

#[test]
fn claim_test_stake_rejects_alternate_token_account() {
    let mut fixture = setup();
    let alternate_account = create_token_account(
        &mut fixture.ctx,
        fixture.claimant.pubkey(),
        fixture.stake_mint,
    );
    let ix = anchor_litesvm::Instruction {
        program_id: demo_faucet::ID,
        accounts: demo_faucet::accounts::ClaimTestStake {
            claimant: fixture.claimant.pubkey(),
            stake_mint: fixture.stake_mint,
            faucet_authority: fixture.faucet_authority,
            claimant_stake_account: alternate_account,
            claim_receipt: fixture.claim_receipt,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: demo_faucet::instruction::ClaimTestStake {}.data(),
    };

    let result = execute_claim(&mut fixture, ix);

    assert!(!result.is_success());
    assert_eq!(read_token(&fixture.ctx, alternate_account).amount, 0);
    assert!(fixture
        .ctx
        .svm
        .get_account(&fixture.claim_receipt)
        .is_none());
}

#[test]
fn claim_test_stake_rejects_wrong_receipt_and_wrong_authority() {
    let mut fixture = setup();
    let wrong_receipt =
        derive_faucet_claim_pda(&demo_faucet::ID, &fixture.stake_mint, &Pubkey::new_unique()).0;
    let wrong_receipt_ix = anchor_litesvm::Instruction {
        program_id: demo_faucet::ID,
        accounts: demo_faucet::accounts::ClaimTestStake {
            claimant: fixture.claimant.pubkey(),
            stake_mint: fixture.stake_mint,
            faucet_authority: fixture.faucet_authority,
            claimant_stake_account: fixture.claimant_stake_ata,
            claim_receipt: wrong_receipt,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: demo_faucet::instruction::ClaimTestStake {}.data(),
    };
    assert!(!execute_claim(&mut fixture, wrong_receipt_ix).is_success());

    let wrong_authority_ix = anchor_litesvm::Instruction {
        program_id: demo_faucet::ID,
        accounts: demo_faucet::accounts::ClaimTestStake {
            claimant: fixture.claimant.pubkey(),
            stake_mint: fixture.stake_mint,
            faucet_authority: Pubkey::new_unique(),
            claimant_stake_account: fixture.claimant_stake_ata,
            claim_receipt: fixture.claim_receipt,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: demo_faucet::instruction::ClaimTestStake {}.data(),
    };
    assert!(!execute_claim(&mut fixture, wrong_authority_ix).is_success());
    assert!(fixture
        .ctx
        .svm
        .get_account(&fixture.claimant_stake_ata)
        .is_none());
}

#[test]
fn claim_test_stake_rejects_wrong_mint_configuration_with_rollback() {
    let mut fixture = setup();
    let bad_mint_keypair = Keypair::new();
    let (bad_faucet_authority, _) =
        derive_faucet_authority_pda(&demo_faucet::ID, &bad_mint_keypair.pubkey());
    let bad_mint = create_mint(
        &mut fixture.ctx,
        &bad_mint_keypair,
        bad_faucet_authority,
        TOKEN_DECIMALS - 1,
    );
    let bad_ata = ata(&fixture.claimant.pubkey(), &bad_mint);
    let bad_receipt =
        derive_faucet_claim_pda(&demo_faucet::ID, &bad_mint, &fixture.claimant.pubkey()).0;

    let ix = anchor_litesvm::Instruction {
        program_id: demo_faucet::ID,
        accounts: demo_faucet::accounts::ClaimTestStake {
            claimant: fixture.claimant.pubkey(),
            stake_mint: bad_mint,
            faucet_authority: bad_faucet_authority,
            claimant_stake_account: bad_ata,
            claim_receipt: bad_receipt,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: demo_faucet::instruction::ClaimTestStake {}.data(),
    };

    let result = execute_claim(&mut fixture, ix);

    assert!(!result.is_success());
    assert!(fixture.ctx.svm.get_account(&bad_ata).is_none());
    assert!(fixture.ctx.svm.get_account(&bad_receipt).is_none());
}
