//! Milestone 19: localnet/devnet setup helper with secret-safe metadata output.

use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use solana_program_pack::Pack;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{read_keypair_file, write_keypair_file, Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::{initialize_mint, mint_to, set_authority},
    state::Mint,
};

const TOKEN_DECIMALS: u8 = 6;
const BASE_UNITS_PER_TOKEN: u64 = 1_000_000;
const REWARD_SUPPLY_TOKENS: u64 = 1_000_000;
const DEVNET_MAX_REWARD_RATE_PER_SLOT: u64 = 100 * BASE_UNITS_PER_TOKEN;
const MIN_PAYER_BALANCE_LAMPORTS: u64 = LAMPORTS_PER_SOL / 2;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetupConfig {
    cluster: String,
    rpc_url: String,
    payer_keypair: String,
    keypair_dir: String,
    pool_id: u64,
    admins: [String; 3],
    max_reward_rate_per_slot_base_units: u64,
    initial_reward_funding_base_units: u64,
}

#[derive(Debug, Serialize)]
struct DeploymentMetadata {
    milestone: u8,
    cluster: String,
    rpc_url_label: String,
    staking_program: String,
    demo_faucet_program: String,
    payer: String,
    upgrade_authority: String,
    pool_id: u64,
    stake_mint: String,
    reward_mint: String,
    reward_treasury_ata: String,
    pool: String,
    pool_authority: String,
    stake_vault: String,
    reward_vault: String,
    faucet_authority: String,
    admins: [String; 3],
    max_reward_rate_per_slot_base_units: u64,
    initial_reward_supply_base_units: u64,
    initial_reward_funding_base_units: u64,
    reward_mint_authority_revoked: bool,
    notes: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse()?;
    let config = SetupConfig::read(&args.config)?;
    let plan = SetupPlan::new(config, args.output, args.mode)?;

    plan.validate(args.mode)?;

    match args.mode {
        Mode::Validate => {
            println!("Milestone 19 config validation passed.");
            println!("cluster: {}", plan.config.cluster);
            println!("payer: {}", plan.payer.pubkey());
            println!("pool: {}", plan.pool);
            println!("stake mint keypair: {}", plan.stake_mint_path.display());
            println!("reward mint keypair: {}", plan.reward_mint_path.display());
        }
        Mode::DryRun => {
            println!("{}", serde_json::to_string_pretty(&plan.metadata())?);
        }
        Mode::Setup => {
            plan.setup()?;
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Validate,
    DryRun,
    Setup,
}

#[derive(Debug)]
struct Args {
    mode: Mode,
    config: PathBuf,
    output: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut mode = None;
        let mut config = PathBuf::from("scripts/devnet/config.local.json");
        let mut output = PathBuf::from("deployments/devnet.json");
        let mut args = std::env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "validate" => mode = Some(Mode::Validate),
                "dry-run" => mode = Some(Mode::DryRun),
                "setup" => mode = Some(Mode::Setup),
                "--config" => {
                    config = PathBuf::from(
                        args.next()
                            .ok_or_else(|| anyhow!("--config requires a path"))?,
                    );
                }
                "--output" => {
                    output = PathBuf::from(
                        args.next()
                            .ok_or_else(|| anyhow!("--output requires a path"))?,
                    );
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => bail!("unknown argument: {other}"),
            }
        }

        Ok(Self {
            mode: mode.unwrap_or(Mode::Validate),
            config,
            output,
        })
    }
}

fn print_help() {
    println!(
        "Milestone 19 deployment helper\n\n\
         Usage:\n\
           cargo run -p deployment_tools -- validate --config <path>\n\
           cargo run -p deployment_tools -- dry-run --config <path>\n\
           cargo run -p deployment_tools -- setup --config <path> --output deployments/devnet.json\n"
    );
}

impl SetupConfig {
    fn read(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        reject_secret_like_config(&raw)?;
        serde_json::from_str(&raw)
            .with_context(|| format!("invalid JSON config {}", path.display()))
    }
}

struct SetupPlan {
    config: SetupConfig,
    output: PathBuf,
    payer: Keypair,
    admin_pubkeys: [Pubkey; 3],
    stake_mint_path: PathBuf,
    reward_mint_path: PathBuf,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
    reward_treasury_ata: Pubkey,
    pool: Pubkey,
    pool_authority: Pubkey,
    stake_vault: Pubkey,
    reward_vault: Pubkey,
    faucet_authority: Pubkey,
}

impl SetupPlan {
    fn new(config: SetupConfig, output: PathBuf, mode: Mode) -> Result<Self> {
        let payer = read_keypair_file(&config.payer_keypair)
            .map_err(|error| anyhow!("failed to read payer keypair path: {error}"))?;
        let admin_pubkeys = parse_admins(&config.admins)?;
        let keypair_dir = PathBuf::from(&config.keypair_dir);
        let stake_mint_path = keypair_dir.join("stake-mint.json");
        let reward_mint_path = keypair_dir.join("reward-mint.json");
        if mode == Mode::Setup {
            fs::create_dir_all(&keypair_dir)?;
        }
        let stake_mint = keypair_pubkey_or_new_address(&stake_mint_path, mode == Mode::Setup)?;
        let reward_mint = keypair_pubkey_or_new_address(&reward_mint_path, mode == Mode::Setup)?;

        let (pool, _) = staking_pool::state::derive_pool_pda(
            &staking_pool::ID,
            &payer.pubkey(),
            config.pool_id,
        );
        let (pool_authority, _) =
            staking_pool::state::derive_pool_authority_pda(&staking_pool::ID, &pool);
        let (faucet_authority, _) =
            demo_faucet::derive_faucet_authority_pda(&demo_faucet::ID, &stake_mint);
        let reward_treasury_ata = get_associated_token_address(&payer.pubkey(), &reward_mint);
        let stake_vault = get_associated_token_address(&pool_authority, &stake_mint);
        let reward_vault = get_associated_token_address(&pool_authority, &reward_mint);

        Ok(Self {
            config,
            output,
            payer,
            admin_pubkeys,
            stake_mint_path,
            reward_mint_path,
            stake_mint,
            reward_mint,
            reward_treasury_ata,
            pool,
            pool_authority,
            stake_vault,
            reward_vault,
            faucet_authority,
        })
    }

    fn validate(&self, mode: Mode) -> Result<()> {
        if !matches!(self.config.cluster.as_str(), "localnet" | "devnet") {
            bail!("cluster must be localnet or devnet");
        }
        if self.config.cluster == "devnet" && self.output != Path::new("deployments/devnet.json") {
            bail!("devnet setup must write to deployments/devnet.json");
        }
        if looks_like_secret_path(&self.output) {
            bail!("output path must be public deployment metadata, not a private path");
        }
        require_distinct_non_default_admins(&self.admin_pubkeys)?;
        if self.config.max_reward_rate_per_slot_base_units > DEVNET_MAX_REWARD_RATE_PER_SLOT {
            bail!("max reward rate exceeds the SPEC devnet maximum");
        }
        if self.config.initial_reward_funding_base_units == 0 {
            bail!("initial_reward_funding_base_units must be positive");
        }
        let max_supply = REWARD_SUPPLY_TOKENS
            .checked_mul(BASE_UNITS_PER_TOKEN)
            .ok_or_else(|| anyhow!("reward supply overflow"))?;
        if self.config.initial_reward_funding_base_units > max_supply {
            bail!("initial funding cannot exceed initial REWARD supply");
        }
        if self.stake_mint == self.reward_mint {
            bail!("stake mint and reward mint must be different");
        }
        if mode == Mode::Setup {
            if self.config.cluster == "devnet"
                && !matches!(
                    self.config.rpc_url.as_str(),
                    "devnet" | "https://api.devnet.solana.com"
                )
            {
                bail!("devnet config may only commit the public devnet RPC label/URL; use Solana CLI config for private RPCs");
            }
            let client = self.client();
            let balance = client
                .get_balance(&self.payer.pubkey())
                .context("failed to fetch payer SOL balance")?;
            if balance < MIN_PAYER_BALANCE_LAMPORTS {
                bail!("payer needs at least 0.5 SOL for setup; current lamports: {balance}");
            }
        }
        Ok(())
    }

    fn setup(&self) -> Result<()> {
        let stake_mint = ensure_keypair(&self.stake_mint_path)?;
        let reward_mint = ensure_keypair(&self.reward_mint_path)?;
        if stake_mint.pubkey() != self.stake_mint || reward_mint.pubkey() != self.reward_mint {
            bail!("mint keypair changed after planning; rerun setup");
        }
        let client = self.client();

        ensure_program_deployed(&client, &staking_pool::ID, "staking_pool")?;
        ensure_program_deployed(&client, &demo_faucet::ID, "demo_faucet")?;
        ensure_mint(
            &client,
            &self.payer,
            &stake_mint,
            &self.faucet_authority,
            false,
        )?;
        ensure_mint(
            &client,
            &self.payer,
            &reward_mint,
            &self.payer.pubkey(),
            true,
        )?;
        ensure_reward_treasury(&client, &self.payer, &self.reward_mint)?;
        ensure_reward_supply_and_revoke(
            &client,
            &self.payer,
            &self.reward_mint,
            self.reward_treasury_ata,
        )?;
        ensure_pool_initialized(&client, self)?;
        ensure_reward_funded(&client, self)?;
        write_metadata(&self.output, &self.metadata())?;

        println!("Milestone 19 setup complete.");
        println!("metadata: {}", self.output.display());
        Ok(())
    }

    fn client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.config.rpc_url.clone(), CommitmentConfig::confirmed())
    }

    fn metadata(&self) -> DeploymentMetadata {
        DeploymentMetadata {
            milestone: 19,
            cluster: self.config.cluster.clone(),
            rpc_url_label: public_rpc_label(&self.config.rpc_url).to_string(),
            staking_program: staking_pool::ID.to_string(),
            demo_faucet_program: demo_faucet::ID.to_string(),
            payer: self.payer.pubkey().to_string(),
            upgrade_authority: self.payer.pubkey().to_string(),
            pool_id: self.config.pool_id,
            stake_mint: self.stake_mint.to_string(),
            reward_mint: self.reward_mint.to_string(),
            reward_treasury_ata: self.reward_treasury_ata.to_string(),
            pool: self.pool.to_string(),
            pool_authority: self.pool_authority.to_string(),
            stake_vault: self.stake_vault.to_string(),
            reward_vault: self.reward_vault.to_string(),
            faucet_authority: self.faucet_authority.to_string(),
            admins: self.config.admins.clone(),
            max_reward_rate_per_slot_base_units: self.config.max_reward_rate_per_slot_base_units,
            initial_reward_supply_base_units: REWARD_SUPPLY_TOKENS * BASE_UNITS_PER_TOKEN,
            initial_reward_funding_base_units: self.config.initial_reward_funding_base_units,
            reward_mint_authority_revoked: true,
            notes: vec![
                "Public metadata only; keypairs and private RPC URLs are intentionally excluded."
                    .to_string(),
                "Direct vault transfers are surplus and are not accounting credit.".to_string(),
                "Educational Devnet deployment; not production-ready.".to_string(),
            ],
        }
    }
}

fn reject_secret_like_config(raw: &str) -> Result<()> {
    let lower = raw.to_ascii_lowercase();
    for needle in [
        "seed phrase",
        "mnemonic",
        "private_key",
        "secretkey",
        "\"secret\"",
        "[0,",
    ] {
        if lower.contains(needle) {
            bail!("config appears to contain secret material: {needle}");
        }
    }
    Ok(())
}

fn looks_like_secret_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == ".private")
}

fn public_rpc_label(rpc_url: &str) -> &str {
    match rpc_url {
        "http://127.0.0.1:8899" | "http://localhost:8899" => "localnet",
        "https://api.devnet.solana.com" | "devnet" => "devnet",
        other => other,
    }
}

fn parse_admins(admins: &[String; 3]) -> Result<[Pubkey; 3]> {
    Ok([
        Pubkey::from_str(&admins[0]).context("invalid admin[0] pubkey")?,
        Pubkey::from_str(&admins[1]).context("invalid admin[1] pubkey")?,
        Pubkey::from_str(&admins[2]).context("invalid admin[2] pubkey")?,
    ])
}

fn require_distinct_non_default_admins(admins: &[Pubkey; 3]) -> Result<()> {
    for admin in admins {
        if *admin == Pubkey::default() {
            bail!("admin keys must not be the default pubkey");
        }
    }
    if admins[0] == admins[1] || admins[0] == admins[2] || admins[1] == admins[2] {
        bail!("admin keys must be distinct");
    }
    Ok(())
}

fn keypair_pubkey_or_new_address(path: &Path, persist_if_missing: bool) -> Result<Pubkey> {
    if path.exists() {
        let keypair = read_keypair_file(path)
            .map_err(|error| anyhow!("failed to read {}: {error}", path.display()))?;
        Ok(keypair.pubkey())
    } else if persist_if_missing {
        Ok(ensure_keypair(path)?.pubkey())
    } else {
        Ok(Keypair::new().pubkey())
    }
}

fn ensure_keypair(path: &Path) -> Result<Keypair> {
    if path.exists() {
        read_keypair_file(path)
            .map_err(|error| anyhow!("failed to read {}: {error}", path.display()))
    } else {
        let keypair = Keypair::new();
        write_keypair_file(&keypair, path)
            .map_err(|error| anyhow!("failed to write {}: {error}", path.display()))?;
        Ok(keypair)
    }
}

fn ensure_program_deployed(client: &RpcClient, program_id: &Pubkey, name: &str) -> Result<()> {
    let account = client
        .get_account(program_id)
        .with_context(|| format!("{name} program is not deployed at {program_id}"))?;
    if !account.executable {
        bail!("{name} account exists but is not executable: {program_id}");
    }
    Ok(())
}

fn ensure_mint(
    client: &RpcClient,
    payer: &Keypair,
    mint: &Keypair,
    mint_authority: &Pubkey,
    temporary_payer_authority: bool,
) -> Result<()> {
    if let Ok(account) = client.get_account(&mint.pubkey()) {
        let unpacked = Mint::unpack(&account.data).context("existing mint account is invalid")?;
        if unpacked.decimals != TOKEN_DECIMALS {
            bail!("existing mint {} does not have six decimals", mint.pubkey());
        }
        return Ok(());
    }

    let rent = client.get_minimum_balance_for_rent_exemption(Mint::LEN)?;
    let payer_pubkey = payer.pubkey();
    let authority = if temporary_payer_authority {
        &payer_pubkey
    } else {
        mint_authority
    };
    let instructions = vec![
        system_instruction::create_account(
            &payer_pubkey,
            &mint.pubkey(),
            rent,
            Mint::LEN as u64,
            &spl_token::ID,
        ),
        initialize_mint(
            &spl_token::ID,
            &mint.pubkey(),
            authority,
            None,
            TOKEN_DECIMALS,
        )?,
    ];
    send(client, instructions, payer, &[mint])?;
    Ok(())
}

fn ensure_reward_treasury(client: &RpcClient, payer: &Keypair, reward_mint: &Pubkey) -> Result<()> {
    let ata = get_associated_token_address(&payer.pubkey(), reward_mint);
    if client.get_account(&ata).is_ok() {
        return Ok(());
    }
    send(
        client,
        vec![create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            reward_mint,
            &spl_token::ID,
        )],
        payer,
        &[],
    )
}

fn ensure_reward_supply_and_revoke(
    client: &RpcClient,
    payer: &Keypair,
    reward_mint: &Pubkey,
    reward_treasury: Pubkey,
) -> Result<()> {
    let account = client.get_account(reward_mint)?;
    let mint = Mint::unpack(&account.data).context("reward mint account is invalid")?;
    let supply = REWARD_SUPPLY_TOKENS * BASE_UNITS_PER_TOKEN;

    if mint.supply == 0 {
        send(
            client,
            vec![mint_to(
                &spl_token::ID,
                reward_mint,
                &reward_treasury,
                &payer.pubkey(),
                &[],
                supply,
            )?],
            payer,
            &[],
        )?;
    } else if mint.supply != supply {
        bail!("reward mint supply is {}, expected {}", mint.supply, supply);
    }

    let refreshed = Mint::unpack(&client.get_account(reward_mint)?.data)?;
    if refreshed.mint_authority.is_some() {
        send(
            client,
            vec![set_authority(
                &spl_token::ID,
                reward_mint,
                None,
                spl_token::instruction::AuthorityType::MintTokens,
                &payer.pubkey(),
                &[],
            )?],
            payer,
            &[],
        )?;
    }
    Ok(())
}

fn ensure_pool_initialized(client: &RpcClient, plan: &SetupPlan) -> Result<()> {
    if client.get_account(&plan.pool).is_ok() {
        return Ok(());
    }

    let accounts = staking_pool::accounts::InitializePool {
        initializer: plan.payer.pubkey(),
        pool: plan.pool,
        pool_authority: plan.pool_authority,
        stake_mint: plan.stake_mint,
        reward_mint: plan.reward_mint,
        stake_vault: plan.stake_vault,
        reward_vault: plan.reward_vault,
        token_program: spl_token::ID,
        associated_token_program: spl_associated_token_account::ID,
        system_program: solana_sdk::system_program::ID,
        rent: solana_sdk::sysvar::rent::ID,
    };
    let data = staking_pool::instruction::InitializePool {
        pool_id: plan.config.pool_id,
        admins: plan.admin_pubkeys,
        max_reward_rate_per_slot: plan.config.max_reward_rate_per_slot_base_units,
    }
    .data();
    send(
        client,
        vec![Instruction {
            program_id: staking_pool::ID,
            accounts: accounts.to_account_metas(None),
            data,
        }],
        &plan.payer,
        &[],
    )
}

fn ensure_reward_funded(client: &RpcClient, plan: &SetupPlan) -> Result<()> {
    if let Ok(account) = client.get_account(&plan.pool) {
        let mut data = account.data.as_slice();
        let pool = staking_pool::state::Pool::try_deserialize(&mut data)
            .context("existing pool account could not be deserialized")?;
        let desired_scaled = u128::from(plan.config.initial_reward_funding_base_units)
            .checked_mul(staking_pool::constants::REWARD_PRECISION)
            .ok_or_else(|| anyhow!("initial funding scale overflow"))?;
        let credited_scaled = pool
            .remaining_reward_budget_scaled
            .checked_add(pool.allocated_liability_scaled)
            .ok_or_else(|| anyhow!("existing pool accounting overflow"))?;
        if credited_scaled >= desired_scaled {
            return Ok(());
        }
    }

    let accounts = staking_pool::accounts::FundRewards {
        source_authority: plan.payer.pubkey(),
        pool: plan.pool,
        source_reward_account: plan.reward_treasury_ata,
        reward_mint: plan.reward_mint,
        reward_vault: plan.reward_vault,
        token_program: spl_token::ID,
    };
    let data = staking_pool::instruction::FundRewards {
        amount: plan.config.initial_reward_funding_base_units,
    }
    .data();
    send(
        client,
        vec![Instruction {
            program_id: staking_pool::ID,
            accounts: accounts.to_account_metas(None),
            data,
        }],
        &plan.payer,
        &[],
    )
}

fn send(
    client: &RpcClient,
    instructions: Vec<Instruction>,
    payer: &Keypair,
    extra_signers: &[&Keypair],
) -> Result<()> {
    let blockhash = client.get_latest_blockhash()?;
    let mut signers = Vec::with_capacity(extra_signers.len() + 1);
    signers.push(payer);
    signers.extend_from_slice(extra_signers);
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &signers,
        blockhash,
    );
    client
        .send_and_confirm_transaction(&transaction)
        .context("transaction failed")?;
    Ok(())
}

fn write_metadata(path: &Path, metadata: &DeploymentMetadata) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(metadata)?;
    reject_secret_like_config(&json)?;
    fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn milestone19_rejects_duplicate_admins() {
        let admin = Pubkey::new_unique();
        let result = require_distinct_non_default_admins(&[admin, admin, Pubkey::new_unique()]);

        assert!(result.is_err());
    }

    #[test]
    fn milestone19_rejects_default_admin() {
        let result = require_distinct_non_default_admins(&[
            Pubkey::new_unique(),
            Pubkey::default(),
            Pubkey::new_unique(),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn milestone19_rejects_secret_like_config() {
        let raw = r#"{"private_key":[0,1,2,3]}"#;

        assert!(reject_secret_like_config(raw).is_err());
    }

    #[test]
    fn milestone19_identifies_private_output_paths() {
        assert!(looks_like_secret_path(Path::new(
            ".private/deployments/devnet.json"
        )));
        assert!(!looks_like_secret_path(Path::new(
            "deployments/devnet.json"
        )));
    }

    #[test]
    fn milestone19_public_rpc_labels_are_stable() {
        assert_eq!(public_rpc_label("https://api.devnet.solana.com"), "devnet");
        assert_eq!(public_rpc_label("http://127.0.0.1:8899"), "localnet");
    }
}
