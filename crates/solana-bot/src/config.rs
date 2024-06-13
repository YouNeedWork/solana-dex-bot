use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "Solana telegram bot",
    about = "A telegram bot that interacts with the Solana blockchain."
)]
pub struct Config {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,
    #[arg(long, env = "TELEGRAM_TOKEN")]
    pub telegram_token: String,
    #[arg(long, env = "RPC_URL")]
    pub rpc_url: String,
    #[arg(long, env = "WS_URL")]
    pub ws_url: Option<String>,
}
