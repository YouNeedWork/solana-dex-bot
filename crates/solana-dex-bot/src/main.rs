use anyhow::Result;
use clap::Parser;
use raydium_library::amm;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::message::Message as SolMessage;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::transaction::Transaction;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    signer::SeedDerivable,
};
use std::str::FromStr;
use teloxide::prelude::*;
use teloxide::types::User;

use bot::SolanaBot;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout as stdout;
use tracing::{error, span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

mod bot;
mod config;
mod jito;

pub fn create_ata_token_or_not(
    funding: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Vec<Instruction> {
    vec![
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            funding,
            owner,
            mint,
            &spl_token::id(),
        ),
    ]
}

pub fn close_account(
    close_account: &Pubkey,
    destination: &Pubkey,
    close_authority: &Pubkey,
) -> Vec<Instruction> {
    vec![spl_token::instruction::close_account(
        &spl_token::id(),
        close_account,
        destination,
        &close_authority,
        &[],
    )
    .unwrap()]
}

pub fn make_compute_budget_ixs(price: u64, max_units: u32) -> Vec<Instruction> {
    vec![
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(price),
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(max_units),
    ]
}

pub fn create_init_token(
    token: &Pubkey,
    seed: &str,
    mint: &Pubkey,
    owner: &Pubkey,
    funding: &Pubkey,
    lamports: u64,
) -> Vec<Instruction> {
    vec![
        solana_sdk::system_instruction::create_account_with_seed(
            funding,
            token,
            owner,
            seed,
            lamports,
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_account(&spl_token::id(), token, mint, owner).unwrap(),
    ]
}

pub fn generate_pub_key(from: &Pubkey, seed: &str) -> Pubkey {
    Pubkey::create_with_seed(from, seed, &spl_token::id()).unwrap()
}

pub struct Swap {
    pre_swap_instructions: Vec<Instruction>,
    post_swap_instructions: Vec<Instruction>,
}

pub async fn handle_token_account(
    swap: &mut Swap,
    client: &RpcClient,
    mint: &Pubkey,
    amount: u64,
    owner: &Pubkey,
    funding: &Pubkey,
) -> Result<Pubkey> {
    // two cases - an account is a token account or a native account (WSOL)
    if (*mint).to_string() == "So11111111111111111111111111111111111111112" {
        let rent = client
            .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
            .await?;
        let lamports = rent + amount;
        let seed = &Keypair::new().pubkey().to_string()[0..32];
        let token = generate_pub_key(owner, seed);
        let mut init_ixs = create_init_token(&token, seed, mint, owner, funding, lamports);
        let mut close_ixs = close_account(&token, owner, owner);
        // swap.signers.push(token);
        swap.pre_swap_instructions.append(&mut init_ixs);
        swap.post_swap_instructions.append(&mut close_ixs);
        Ok(token)
    } else {
        let token = &spl_associated_token_account::get_associated_token_address(owner, mint);
        let mut ata_ixs = create_ata_token_or_not(funding, mint, owner);
        swap.pre_swap_instructions.append(&mut ata_ixs);
        Ok(*token)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new OpenTelemetry trace pipeline that prints to stdout
    let provider = TracerProvider::builder()
        .with_simple_exporter(stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("readme_example");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(telemetry);
    //subscriber.into();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    //let _cfg = config::Config::parse();
    // Spans will be sent to the configured OpenTelemetry exporter
    let root = span!(tracing::Level::TRACE, "app_start", work_units = 2);
    let _enter = root.enter();

    error!("This event will be logged in the root span.");

    //let key = Keypair::new();
    let wallet = Keypair::from_seed(&[
        118, 206, 164, 217, 88, 74, 225, 36, 231, 186, 155, 160, 221, 19, 71, 28, 253, 155, 196,
        38, 231, 56, 108, 80, 34, 160, 46, 147, 98, 213, 233, 119,
    ])
    .unwrap();

    println!("私钥: {:?}", wallet.secret());
    println!("公钥: {:?}", wallet.pubkey());

    let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
    //let client = RpcClient::new("https://convincing-black-wish.solana-devnet.quiknode.pro/6048f8fac89bf3526d3b636b4d4298af84593507/".to_string());
    let balances = client.get_balance(&wallet.pubkey()).await.unwrap();

    println!("balance: {:?} SOL ", balances as f64 / 1_000_000_000 as f64);
    // config params

    let amm_program = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    let amm_pool_id = Pubkey::from_str("H6iiLoyfQg4GXATaRUwgJqTj7a7NStKjKEiDPqafvrMg")?;
    let input_token_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let output_token_mint = Pubkey::from_str("E3ZELac8ywEmt5WL5WVncrCXPePSoZuwaQ7rqJDTxs8M")?;
    let slippage_bps = 1000u64; // 0.5%
    let amount_specified = 100000u64;
    let swap_base_in = true;

    // load amm keys
    let amm_keys = amm::utils::load_amm_keys(&client, &amm_program, &amm_pool_id).await?;
    println!("{:?}", amm_keys);
    // load market keys
    let market_keys =
        amm::openbook::get_keys_for_market(&client, &amm_keys.market_program, &amm_keys.market)
            .await
            .expect("market get failed");

    println!("{:?}", amm_keys);
    // calculate amm pool vault with load data at the same time or use simulate to calculate
    let result = amm::calculate_pool_vault_amounts(
        &client,
        &amm_program,
        &amm_pool_id,
        &amm_keys,
        &market_keys,
        //amm::utils::CalculateMethod::CalculateWithLoadAccount,
        amm::utils::CalculateMethod::Simulate(wallet.pubkey()),
    )
    .await?;
    println!("{:?}", result);

    let direction = if input_token_mint == amm_keys.amm_coin_mint
        && output_token_mint == amm_keys.amm_pc_mint
    {
        amm::utils::SwapDirection::Coin2PC
    } else {
        amm::utils::SwapDirection::PC2Coin
    };

    let mut other_amount_threshold = amm::swap_with_slippage(
        result.pool_pc_vault_amount,
        result.pool_coin_vault_amount,
        result.swap_fee_numerator,
        result.swap_fee_denominator,
        direction,
        amount_specified,
        swap_base_in,
        slippage_bps,
    )?;

    println!("other_amount_threshold: {:?}", other_amount_threshold);

    let mut swap = Swap {
        pre_swap_instructions: vec![],
        post_swap_instructions: vec![],
    };

    let user_source = handle_token_account(
        &mut swap,
        &client,
        &input_token_mint,
        other_amount_threshold,
        &wallet.pubkey(),
        &wallet.pubkey(),
    )
    .await?;

    let user_destination = handle_token_account(
        &mut swap,
        &client,
        &output_token_mint,
        0,
        &wallet.pubkey(),
        &wallet.pubkey(),
    )
    .await?;

    // build swap instruction
    let build_swap_instruction = amm::instructions::swap(
        &amm_program,
        &amm_keys,
        &market_keys,
        &wallet.pubkey(),
        &user_source,
        &user_destination,
        amount_specified,
        other_amount_threshold,
        swap_base_in,
    )?;

    println!("{:?}", build_swap_instruction);

    let blockhash = client.get_latest_blockhash().await?;
    let txs = [
        make_compute_budget_ixs(0, 300_000),
        swap.pre_swap_instructions.clone(),
        vec![build_swap_instruction],
        swap.post_swap_instructions.clone(),
    ]
    .concat();
    dbg!(&txs);
    let mut message = SolMessage::new(&txs, Some(&wallet.pubkey()));
    message.recent_blockhash = blockhash;

    let mut tx = Transaction::new_unsigned(message);

    //    info!("Simulation: {}", serde_json::to_string_pretty(&sim_res)?);

    //tx.sign(&wallet, blockhash);
    let signature = wallet.sign_message(&tx.message_data());
    tx.replace_signatures(&[(wallet.pubkey(), signature)])?;

    //let searcher_client = "";
    let tx = VersionedTransaction::from(tx);
    let sim_res = client.simulate_transaction(&tx).await?;
    println!("sim_res {:?}", &sim_res);
    //jito::send_swap_tx([tx], 50000, &wallet.pubkey(), searcher_client, &client);
    //let res = client.send_and_confirm_transaction(&tx).await?;

    //println!("{:?}", res);

    /*
        let bot = Bot::from_env();
        teloxide::repl(bot, |bot: Bot, msg: Message| async move {
            bot.send_dice(msg.chat.id).await?;
            Ok(())
        })
        .await;
    */
    //teloxide::enable_logging!();

    log::info!("Starting bot...");

    let bot = SolanaBot::new(std::env::var("TELOXIDE_TOKEN").unwrap()).unwrap();
    bot.run().await
}

fn extract_user(msg: &Message) -> Option<User> {
    match &msg.kind {
        teloxide::types::MessageKind::Common(mc) => mc.from.clone(),
        _ => None,
    }
}
