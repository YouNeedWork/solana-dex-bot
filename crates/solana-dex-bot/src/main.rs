use anyhow::{format_err, Result};
use clap::Parser;
use raydium_library::amm;
use raydium_library::amm::SwapDirection;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    signer::SeedDerivable,
};
use std::str::FromStr;
use teloxide::prelude::*;
mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let _cfg = config::Config::parse();

    //let key = Keypair::new();
    let wallet = Keypair::from_seed(&[
        118, 206, 164, 217, 88, 74, 225, 36, 231, 186, 155, 160, 221, 19, 71, 28, 253, 155, 196,
        38, 231, 56, 108, 80, 34, 160, 46, 147, 98, 213, 233, 119,
    ])
    .unwrap();

    println!("私钥: {:?}", wallet.secret());
    println!("公钥: {:?}", wallet.pubkey());

    //let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
    let client = RpcClient::new("https://convincing-black-wish.solana-devnet.quiknode.pro/6048f8fac89bf3526d3b636b4d4298af84593507/".to_string());
    let balances = client.get_balance(&wallet.pubkey()).unwrap();

    println!("balance: {:?} SOL ", balances as f64 / 1_000_000_000 as f64);
    // config params

    let amm_program = Pubkey::from_str("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8")?;
    let amm_pool_id = Pubkey::from_str("BbZjQanvSaE9me4adAitmTTaSgASvzaVignt4HRSM7ww")?;
    let input_token_mint = Pubkey::from_str("GfmdKWR1KrttDsQkJfwtXovZw9bUBHYkPAEwB6wZqQvJ")?;
    let output_token_mint = Pubkey::from_str("2SiSpNowr7zUv5ZJHuzHszskQNaskWsNukhivCtuVLHo")?;
    let slippage_bps = 50u64; // 0.5%
    let amount_specified = 2000_000000u64;
    let swap_base_in = false;

    // load amm keys
    let amm_keys = raydium_library::amm::utils::load_amm_keys(&client, &amm_program, &amm_pool_id)?;
    println!("{:?}", amm_keys);
    // load market keys
    let market_keys = raydium_library::amm::openbook::get_keys_for_market(
        &client,
        &amm_keys.market_program,
        &amm_keys.market,
    )?;

    println!("{:?}", amm_keys);
    // calculate amm pool vault with load data at the same time or use simulate to calculate
    let result = amm::calculate_pool_vault_amounts(
        &client,
        &amm_program,
        &amm_pool_id,
        &amm_keys,
        &market_keys,
        amm::utils::CalculateMethod::Simulate(wallet.pubkey()),
    )?;

    let direction = if input_token_mint == amm_keys.amm_coin_mint
        && output_token_mint == amm_keys.amm_pc_mint
    {
        raydium_library::amm::utils::SwapDirection::Coin2PC
    } else {
        raydium_library::amm::utils::SwapDirection::PC2Coin
    };

    let other_amount_threshold = raydium_library::amm::swap_with_slippage(
        result.pool_pc_vault_amount,
        result.pool_coin_vault_amount,
        result.swap_fee_numerator,
        result.swap_fee_denominator,
        direction,
        amount_specified,
        swap_base_in,
        slippage_bps,
    )?;

    // build swap instruction
    let build_swap_instruction = amm::instructions::swap(
        &amm_program,
        &amm_keys,
        &market_keys,
        &wallet.pubkey(),
        &spl_associated_token_account::get_associated_token_address(
            &wallet.pubkey(),
            &input_token_mint,
        ),
        &spl_associated_token_account::get_associated_token_address(
            &wallet.pubkey(),
            &output_token_mint,
        ),
        amount_specified,
        other_amount_threshold,
        swap_base_in,
    )?;

    let bot = Bot::from_env();
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;

    Ok(())
}
