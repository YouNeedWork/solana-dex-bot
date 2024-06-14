#[macro_use]
extern crate rust_i18n;

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_core::trade::Trade;
use solana_sdk::{
    signature::{Keypair, Signer},
    signer::SeedDerivable,
};

use rust_i18n::t;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

mod bot;
mod config;

rust_i18n::i18n!("locales");

#[tokio::main]
async fn main() -> Result<()> {
    //let key = Keypair::new();
    let wallet = Keypair::from_seed(&[
        118, 206, 164, 217, 88, 74, 225, 36, 231, 186, 155, 160, 221, 19, 71, 28, 253, 155, 196,
        38, 231, 56, 108, 80, 34, 160, 46, 147, 98, 213, 233, 119,
    ])
    .unwrap();

    println!("私钥: {:?}", wallet.secret());
    println!("私钥: {:?}", wallet.to_base58_string());
    println!("公钥: {:?}", wallet.pubkey());

    let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
    let balances = client.get_balance(&wallet.pubkey()).await.unwrap();
    println!("balance: {:?} SOL ", balances as f64 / 1_000_000_000 as f64);

    let input_token_mint = Pubkey::from_str("E3ZELac8ywEmt5WL5WVncrCXPePSoZuwaQ7rqJDTxs8M")?;
    let trade = Trade::new(wallet, client);
    let amount = trade.get_spl_balance(&input_token_mint).await?;

    println!("balance: {:?} ", amount);

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

    Ok(())
}
