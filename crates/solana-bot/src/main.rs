use anyhow::Result;
use clap::Parser;
use config::Config;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use rust_i18n::t;

mod bot;
mod config;

rust_i18n::i18n!("locales");

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    tracing_subscriber::fmt::init();

    let manager = ConnectionManager::<PgConnection>::new(config.database_url);

    let db = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    /*
    let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
    let balances = client.get_balance(&wallet.pubkey()).await.unwrap();

    println!("balance: {:?} SOL ", balances as f64 / 1_000_000_000 as f64);

    let input_token_mint = Pubkey::from_str("E3ZELac8ywEmt5WL5WVncrCXPePSoZuwaQ7rqJDTxs8M")?;
    let trade = Trade::new(wallet, client);
    let amount = trade.get_spl_balance(&input_token_mint).await?;

    println!("balance: {:?} ", amount);
    */
    println!("{}", t!("hello"));
    println!("{}", t!("hello", locale = "zh-CN"));

    /*
    trade
        .swap(
            &"E3ZELac8ywEmt5WL5WVncrCXPePSoZuwaQ7rqJDTxs8M",
            solana_core::constants::SOLANA_PROGRAM_ID,
            amount,
            80,
            5000,
        )
        .await?;
    */

    let app_state = bot::AppState::new();
    let bot = bot::SolanaBot::new(config.telegram_token, db, app_state)?;
    bot.run().await?;
    Ok(())
}
