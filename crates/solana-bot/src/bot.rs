use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_core::constants;
use solana_core::trade::Trade;
use solana_program::pubkey::Pubkey;
use solana_sdk::signer::{keypair::Keypair, Signer};
use std::default;
use std::error::Error;
use std::str::FromStr;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::types::Me;
use teloxide::types::ParseMode;
use teloxide::utils::command::BotCommands;
use teloxide::Bot;
use tracing::info;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use solana_core::dexscreen;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use models::wallet::Wallet;

type PGPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct User {
    pub uid: i64,
    pub username: String,
    pub wallet_address: Pubkey,
    pub private_key: String,
    pub tip: i64,
    pub slippage: i64,
}

pub struct AppState {
    pub users: HashMap<i64, User>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            users: HashMap::new(),
        }
    }
}

pub struct SolanaBot {
    admin_id: i64,
    bot: Bot,
    db: PGPool,
    app_state: AppState,
}

impl SolanaBot {
    pub fn new(token: String, db: PGPool, app_state: AppState) -> Result<Self> {
        let bot = teloxide::Bot::new(token);

        Ok(Self {
            bot,
            admin_id: 0,
            db,
            app_state,
        })
    }

    pub async fn run(self) -> Result<()> {
        let SolanaBot {
            bot,
            admin_id: _,
            db,
            app_state,
        } = self;

        let db = Arc::new(db);
        let app_state = Arc::new(RwLock::new(app_state));

        let msg_db = db.clone();
        let app_state1 = app_state.clone();

        let handler =
            dptree::entry()
                .branch(Update::filter_message().endpoint(move |bot, msg, me| {
                    message_handler(bot, msg, me, msg_db.clone(), app_state1.clone())
                }))
                .branch(Update::filter_callback_query().endpoint(move |bot, q| {
                    callback_handler(bot, q, db.clone(), app_state.clone())
                }));

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Help,
    Start,
    Menu,
    Seting,
    Lang,
}

pub async fn menu(
    bot: Bot,
    id: ChatId,
    db: Arc<PGPool>,
    app_state: Arc<RwLock<AppState>>,
) -> Result<()> {
    let user = fetch_user(&bot, id.0 as i64, db, app_state).await?;

    let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
    let trade = Trade::new(Keypair::from_base58_string(&user.private_key), client);

    let amount = trade
        .get_balance()
        .await
        .map(|a| a as f64 / 1_000_000_000 as f64)
        .unwrap_or(0.0);

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Buy/Sell".to_string(), "BuySell".to_string()),
            InlineKeyboardButton::callback("Assets".to_string(), "Assets".to_string()),
        ],
        vec![InlineKeyboardButton::callback(
            "Help".to_string(),
            "help".to_string(),
        )],
    ]);

    let message_text = format!(
        "
Welcome to edgeX Bot!

Introducing a cutting-edge bot built exclusively for Traders. Trade any token instantly at the moment it launches!

Here's your Solana wallet address linked to your Telegram account. To start trading, deposit SOL to your wallet address and dive into trading.

Wallet: `{}`
Balance: {} SOL

✅Send CA to start trading tokens.
",
        &user.wallet_address.to_string(),
        amount
    );

    bot.send_message(id, message_text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Markdown)
        .await?;

    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
    db: Arc<PGPool>,
    app_state: Arc<RwLock<AppState>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            Ok(Command::Start) => {
                menu(bot, msg.chat.id, db, app_state).await?;
            }
            Ok(Command::Menu) => {
                menu(bot, msg.chat.id, db, app_state).await?;
            }
            Ok(Command::Seting) => {}
            Ok(Command::Lang) => {}
            Err(_) => {
                info!(text = &text, "Got text");

                if let Ok(token_in) = Pubkey::from_str(text) {
                    info!(token_in=?token_in.to_string(), "recver token wait to trade");
                    let user = fetch_user(&bot, msg.chat.id.0 as i64, db, app_state).await?;

                    let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
                    let trade = Trade::new(Keypair::from_base58_string(&user.private_key), client);

                    let amount = trade
                        .get_balance()
                        .await
                        .map(|a| a as f64 / 1_000_000_000 as f64)
                        .unwrap_or(0.0);

                    let slp_amount = trade.get_spl_balance(&token_in).await.unwrap_or_default();
                    info!(balance=?amount,token_in=slp_amount, "recver token wait to trade");

                    if let Ok(dex_info) = dexscreen::search(text).await {
                        if dex_info.pairs.is_empty() {
                            let message_text = "Token Address Not found";
                            bot.send_message(msg.chat.id, message_text).await?;
                            return Ok(());
                        }

                        let pair = dex_info.pairs.first().unwrap();

                        info!(dex_info=?dex_info, "found ca");
                        let message_text = format!(
                            "
📌[{}](https://dexscreener.com/solana/{})
{}

💳Wallet: 
|——Balance: {} SOL ($0)
|——Holding:{} SOL ($0)
|___PnL: 0%🚀🚀

💵Trade: 
|——Market Cap: 2.1M
|——Price: 0.0021
|___PepeBoost Buyers: 516

🔍Security:
|——Renounced✅ LP Burnt✅ Freeze revoked✅
|___Top 10 : 18%

📝LP: HOLDY-SOL
|——🟢 Trading opened
|——   Created 0d 7h 27m ago
|___  Liquidity: 636.99 SOL(0.08%)

📲Links:
",
                            pair.base_token.name,
                            pair.pair_address,
                            pair.base_token.address,
                            amount,
                            slp_amount
                        );

                        let keyboard = InlineKeyboardMarkup::new(vec![
                            vec![
                                InlineKeyboardButton::callback(
                                    "Buy 0.01".to_string(),
                                    "Buy1 ".to_string() + &pair.base_token.address,
                                ),
                                InlineKeyboardButton::callback(
                                    "Buy 0.1".to_string(),
                                    "Buy10".to_string(),
                                ),
                                InlineKeyboardButton::callback(
                                    "Buy 1".to_string(),
                                    "Buy100".to_string(),
                                ),
                            ],
                            vec![
                                InlineKeyboardButton::callback(
                                    "Sell 25%".to_string(),
                                    "Sell25".to_string(),
                                ),
                                InlineKeyboardButton::callback(
                                    "Sell 50%".to_string(),
                                    "Sell50".to_string(),
                                ),
                                InlineKeyboardButton::callback(
                                    "Sell 75%".to_string(),
                                    "Sell75".to_string(),
                                ),
                            ],
                            vec![InlineKeyboardButton::callback(
                                "Sell 100%".to_string(),
                                "Sell100".to_string(),
                            )],
                        ]);

                        bot.send_message(msg.chat.id, message_text)
                            .reply_markup(keyboard)
                            .parse_mode(ParseMode::Markdown)
                            .await?;
                    } else {
                        let message_text = format!("Token Not found");
                        bot.send_message(msg.chat.id, message_text).await?;
                    }
                }

                //let trade = Trade::new(wallet, client);
                //let amount = trade.get_spl_balance(&input_token_mint).await?;
            }
        }
    }

    Ok(())
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    db: Arc<PGPool>,
    app_state: Arc<RwLock<AppState>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(chose) = q.data {
        log::info!("You chose: {}", chose);
        bot.answer_callback_query(q.id).await?;

        let com = chose.split(" ").collect::<Vec<&str>>();

        let action = com.first().unwrap();

        let text = match *action {
            "Buy1" => {
                info!("buy 0.01");
                let user = fetch_user(&bot, q.from.id.0 as i64, db, app_state)
                    .await
                    .unwrap();

                let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
                let trade = Trade::new(Keypair::from_base58_string(&user.private_key), client);

                let amount = trade
                    .get_balance()
                    .await
                    .map(|a| a as f64 / 1_000_000_000 as f64)
                    .unwrap_or(0.0);

                if amount < 0.01 {
                    "Insufficient balance"
                } else {
                    if let Ok(_) = trade
                        .swap(
                            com.get(1).unwrap(),
                            constants::SOLANA_PROGRAM_ID,
                            10000000,
                            user.slippage as u64,
                            user.tip as u64,
                        )
                        .await
                    {
                        "Swap success"
                    } else {
                        "Swap failed"
                    }
                }
            }
            "Buy10" => {
                info!("buy 0.1");
                let user = fetch_user(&bot, q.from.id.0 as i64, db, app_state)
                    .await
                    .unwrap();

                let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
                let trade = Trade::new(Keypair::from_base58_string(&user.private_key), client);

                let amount = trade
                    .get_balance()
                    .await
                    .map(|a| a as f64 / 1_000_000_000 as f64)
                    .unwrap_or(0.0);

                if amount < 0.1 {
                    "Insufficient balance"
                } else {
                    if let Ok(_) = trade
                        .swap(
                            com.get(1).unwrap(),
                            constants::SOLANA_PROGRAM_ID,
                            100000000,
                            user.slippage as u64,
                            user.tip as u64,
                        )
                        .await
                    {
                        "Swap success"
                    } else {
                        "Swap failed"
                    }
                }
            }
            "Buy100" => {
                info!("buy 1");
                let user = fetch_user(&bot, q.from.id.0 as i64, db, app_state)
                    .await
                    .unwrap();

                let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
                let trade = Trade::new(Keypair::from_base58_string(&user.private_key), client);

                let amount = trade
                    .get_balance()
                    .await
                    .map(|a| a as f64 / 1_000_000_000 as f64)
                    .unwrap_or(0.0);

                if amount < 1.0 {
                    "Insufficient balance"
                } else {
                    if let Ok(_) = trade
                        .swap(
                            com.get(1).unwrap(),
                            constants::SOLANA_PROGRAM_ID,
                            1_000_000_000,
                            user.slippage as u64,
                            user.tip as u64,
                        )
                        .await
                    {
                        "Swap success"
                    } else {
                        "Swap failed"
                    }
                }
            }
            "Sell25" => {
                info!("sell 25%");
                "Sell 25%"
            }
            "Sell50" => {
                info!("sell 50%");
                "Sell 50%"
            }
            "Sell75" => {
                info!("sell 75%");
                "Sell 75%"
            }
            "Sell100" => {
                info!("sell 100%");
                "Sell 100%"
            }
            _ => "Not found action",
        };

        // Edit text of the message to which the buttons were attached
        if let Some(Message { id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }
    }

    Ok(())
}

async fn fetch_user(
    bot: &Bot,
    user_id: i64,
    db: Arc<PGPool>,
    app_state: Arc<RwLock<AppState>>,
) -> Result<User> {
    let mut app_state = app_state.write().await;

    if app_state.users.contains_key(&user_id) {
        return Ok(app_state.users.get(&user_id).unwrap().clone());
    } else {
        //query default
        match Wallet::fetch_default(&mut db.get().unwrap(), user_id) {
            Ok(wallet) => {
                let user = User {
                    uid: user_id,
                    username: "NewUser".to_string(), // You may want to fetch the actual username from Telegram API
                    wallet_address: Pubkey::from_str(&wallet.wallet_address).unwrap(),
                    private_key: wallet.private_key.clone(),
                    slippage: wallet.slippage.into(),
                    tip: wallet.tip.into(),
                };

                app_state.users.insert(user_id, user.clone());
                Ok(user)
            }
            Err(_) => {
                let keypair = generate_wallet().await?;

                let new_user = User {
                    uid: user_id,
                    username: "NewUser".to_string(), // You may want to fetch the actual username from Telegram API
                    wallet_address: keypair.pubkey(),
                    private_key: keypair.to_base58_string(),
                    slippage: 800,
                    tip: 500000,
                };

                let wallet = Wallet::new(
                    keypair.to_base58_string(),
                    keypair.pubkey().to_string(),
                    user_id,
                    true,
                );

                wallet.create(&mut db.get().unwrap())?;

                // Update the in-memory state
                app_state.users.insert(user_id, new_user.clone());

                return Ok(new_user);
            }
        }
    }
}

async fn generate_wallet() -> Result<Keypair> {
    let mut rng = rand::rngs::OsRng;
    let keypair = Keypair::generate(&mut rng);
    Ok(keypair)
}

async fn buy() -> Result<()> {
    Ok(())
}
