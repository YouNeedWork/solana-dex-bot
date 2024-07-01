use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "Solana telegram bot",
    about = "A telegram bot that interacts with the Solana blockchain."
)]
pub struct Config {
    #[arg(long, env = "RPC_URL")]
    pub rpc_url: String,
    #[arg(long, env = "RABBIT_MQ_URL")]
    pub rabbit_mq_url: String,
    #[arg(long, env = "WS_URL")]
    pub ws_url: Option<String>,
}
