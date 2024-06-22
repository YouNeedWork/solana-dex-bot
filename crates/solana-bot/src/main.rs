use anyhow::Result;
use clap::Parser;
use config::Config;
use diesel::{pg::PgConnection, r2d2::ConnectionManager};

use solana_bot::{bot, config};

rust_i18n::i18n!("locales");

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::parse();

    let manager = ConnectionManager::<PgConnection>::new(config.database_url);

    let db = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    //TODO: example for i18n usage todo for transfer bot interface
    /*
    println!("{}", t!("hello"));
    println!("{}", t!("hello", locale = "zh-CN"));
    */

    let app_state = bot::AppState::new();
    let bot = bot::SolanaBot::new(config.telegram_token, db, app_state)?;
    bot.run().await?;
    Ok(())
}
