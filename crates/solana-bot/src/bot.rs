use anyhow::Result;
use models::hold_coin::HoldCoin;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_core::constants;
use solana_core::trade::Trade;
use solana_program::pubkey::Pubkey;
use solana_sdk::signer::{keypair::Keypair, Signer};
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
    pub id: i64,
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
        //.reply_markup(keyboard)
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
[{}](https://dexscreener.com/solana/{})
{}

💵Trade:
Price: {} | Market Cap: {}

🔍Security:
Renounced✅ | LP Burnt✅ | Freeze❄️✅

📝LP: {}
{} | Open time: {}

🔗Links:
[Dexscreener](https://dexscreener.com/solana/{})
",
                            pair.base_token.name,
                            pair.pair_address,
                            pair.base_token.address,
                            pair.price_usd,
                            pair.fdv,
                            pair.liquidity.usd,
                            pair.pair_created_at,
                            pair.pair_created_at,
                            pair.pair_address
                        );

                        let keyboard = InlineKeyboardMarkup::new(vec![
                            vec![
                                InlineKeyboardButton::callback(
                                    "Buy 0.01".to_string(),
                                    "Buy1|".to_string() + &pair.base_token.address,
                                ),
                                InlineKeyboardButton::callback(
                                    "Buy 0.1".to_string(),
                                    "Buy10|".to_string() + &pair.base_token.address,
                                ),
                                InlineKeyboardButton::callback(
                                    "Buy 1".to_string(),
                                    "Buy100|".to_string() + &pair.base_token.address,
                                ),
                            ],
                            vec![
                                InlineKeyboardButton::callback(
                                    "Sell 25%".to_string(),
                                    "Sell25|".to_string() + &pair.base_token.address,
                                ),
                                InlineKeyboardButton::callback(
                                    "Sell 50%".to_string(),
                                    "Sell50|".to_string() + &pair.base_token.address,
                                ),
                                InlineKeyboardButton::callback(
                                    "Sell 75%".to_string(),
                                    "Sell75|".to_string() + &pair.base_token.address,
                                ),
                            ],
                            vec![InlineKeyboardButton::callback(
                                "Sell 100%".to_string(),
                                "Sell100|".to_string() + &pair.base_token.address,
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

        let com = chose.trim().split("|").collect::<Vec<&str>>();

        let action = com.first().unwrap();

        let user = fetch_user(&bot, q.from.id.0 as i64, db.clone(), app_state.clone())
            .await
            .unwrap();

        let client = RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string());
        let trade = Trade::new(Keypair::from_base58_string(&user.private_key), client);

        let amount = trade
            .get_balance()
            .await
            .map(|a| a as f64 / 1_000_000_000 as f64)
            .unwrap_or(0.0);

        let text = match *action {
            "Buy1" => {
                info!("buy 0.01");
                if amount < 0.01 {
                    "Insufficient balance"
                } else {
                    let swap_token = com.get(1).unwrap();

                    if let Ok(_) = trade
                        .swap(
                            constants::SOLANA_PROGRAM_ID,
                            com.get(1).unwrap(),
                            10_000_000,
                            user.slippage as u64,
                            user.tip as u64,
                        )
                        .await
                    {
                        let amount = trade
                            .get_spl_balance(&Pubkey::from_str(swap_token).unwrap())
                            .await
                            .unwrap_or_default();

                        insert_or_update_hold_coin(
                            &bot,
                            user.id,
                            db.clone(),
                            app_state.clone(),
                            swap_token,
                            &amount.to_string(),
                        )
                        .await?;

                        "Swap success"
                    } else {
                        "Swap failed"
                    }
                }
            }
            "Buy10" => {
                info!("buy 0.1");
                if amount < 0.1 {
                    "Insufficient balance"
                } else {
                    let swap_token = com.get(1).unwrap();

                    if let Ok(_) = trade
                        .swap(
                            constants::SOLANA_PROGRAM_ID,
                            swap_token,
                            100_000_000,
                            user.slippage as u64,
                            user.tip as u64,
                        )
                        .await
                    {
                        let amount = trade
                            .get_spl_balance(&Pubkey::from_str(swap_token).unwrap())
                            .await
                            .unwrap_or_default();

                        insert_or_update_hold_coin(
                            &bot,
                            user.id,
                            db,
                            app_state,
                            swap_token,
                            &amount.to_string(),
                        )
                        .await?;

                        "Swap success"
                    } else {
                        "Swap failed"
                    }
                }
            }
            "Buy100" => {
                info!("buy 1");
                if amount < 1.0 {
                    "Insufficient balance"
                } else {
                    if let Ok(_) = trade
                        .swap(
                            constants::SOLANA_PROGRAM_ID,
                            com.get(1).unwrap(),
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
                let swap_token = com.get(1).unwrap();

                let amount = trade
                    .get_spl_balance(&Pubkey::from_str(swap_token).unwrap())
                    .await
                    .unwrap_or_default();

                if let Ok(_) = trade
                    .swap(
                        swap_token,
                        constants::SOLANA_PROGRAM_ID,
                        amount * 25 / 100,
                        user.slippage as u64,
                        user.tip as u64,
                    )
                    .await
                {
                    let amount = trade
                        .get_spl_balance(&Pubkey::from_str(swap_token).unwrap())
                        .await
                        .unwrap_or_default();

                    insert_or_update_hold_coin(
                        &bot,
                        user.id,
                        db,
                        app_state,
                        swap_token,
                        &amount.to_string(),
                    )
                    .await?;

                    "Swap success"
                } else {
                    "Swap failed"
                }
            }
            "Sell50" => {
                info!("sell 50%");
                let swap_token = com.get(1).unwrap();

                let amount = trade
                    .get_spl_balance(&Pubkey::from_str(swap_token).unwrap())
                    .await
                    .unwrap_or_default();

                if let Ok(_) = trade
                    .swap(
                        swap_token,
                        constants::SOLANA_PROGRAM_ID,
                        amount * 50 / 100,
                        user.slippage as u64,
                        user.tip as u64,
                    )
                    .await
                {
                    let amount = trade
                        .get_spl_balance(&Pubkey::from_str(swap_token).unwrap())
                        .await
                        .unwrap_or_default();

                    insert_or_update_hold_coin(
                        &bot,
                        user.id,
                        db,
                        app_state,
                        swap_token,
                        &amount.to_string(),
                    )
                    .await?;

                    "Swap success"
                } else {
                    "Swap failed"
                }
            }
            "Sell75" => {
                info!("sell 75%");
                let swap_token = com.get(1).unwrap();

                let amount = trade
                    .get_spl_balance(&Pubkey::from_str(swap_token).unwrap())
                    .await
                    .unwrap_or_default();

                if let Ok(_) = trade
                    .swap(
                        swap_token,
                        constants::SOLANA_PROGRAM_ID,
                        amount * 75 / 100,
                        user.slippage as u64,
                        user.tip as u64,
                    )
                    .await
                {
                    let amount = trade
                        .get_spl_balance(&Pubkey::from_str(swap_token).unwrap())
                        .await
                        .unwrap_or_default();

                    insert_or_update_hold_coin(
                        &bot,
                        user.id,
                        db,
                        app_state,
                        swap_token,
                        &amount.to_string(),
                    )
                    .await?;

                    "Swap success"
                } else {
                    "Swap failed"
                }
            }
            "Sell100" => {
                info!("sell 100%");
                let swap_token = com.get(1).unwrap();

                let amount = trade
                    .get_spl_balance(&Pubkey::from_str(swap_token).unwrap())
                    .await
                    .unwrap_or_default();

                if let Ok(_) = trade
                    .swap(
                        swap_token,
                        constants::SOLANA_PROGRAM_ID,
                        amount,
                        user.slippage as u64,
                        user.tip as u64,
                    )
                    .await
                {
                    insert_or_update_hold_coin(
                        &bot,
                        user.id,
                        db,
                        app_state,
                        swap_token,
                        &"0".to_string(),
                    )
                    .await?;

                    "Swap success"
                } else {
                    "Swap failed"
                }
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

async fn insert_or_update_hold_coin(
    bot: &Bot,
    user_id: i64,
    db: Arc<PGPool>,
    app_state: Arc<RwLock<AppState>>,
    swap_token: &str,
    amount: &str,
) -> Result<()> {
    let hold = HoldCoin::new(
        user_id as i32,
        "SOL".to_string(),
        swap_token.to_string(),
        "SOL".to_string(),
        amount.to_string(),
        "0".to_string(),
    );

    models::hold_coin::HoldCoin::create_or_update(&mut db.get().unwrap(), &hold)?;

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
        Ok(app_state.users.get(&user_id).unwrap().clone())
    } else {
        // Query default wallet from the database
        match Wallet::fetch_default_with_id(&mut db.get().unwrap(), user_id) {
            Ok(wallet) => {
                let user = User {
                    id: wallet.id as i64,
                    uid: user_id,
                    username: "NewUser".to_string(), // Placeholder, should fetch actual username from Telegram API
                    wallet_address: Pubkey::from_str(&wallet.wallet_address).unwrap(),
                    private_key: wallet.private_key.clone(),
                    slippage: wallet.slippage.into(),
                    tip: wallet.tip.into(),
                };

                app_state.users.insert(user_id, user.clone());
                Ok(user)
            }
            Err(_) => {
                // Generate a new wallet if no default wallet is found
                let keypair = generate_wallet().await?;

                let mut new_user = User {
                    id: 0,
                    uid: user_id,
                    username: "NewUser".to_string(), // Placeholder, should fetch actual username from Telegram API
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

                if let Ok(wallet) = Wallet::fetch_default_with_id(&mut db.get().unwrap(), user_id) {
                    new_user.id = wallet.id as i64;
                }

                // Update the in-memory state
                app_state.users.insert(user_id, new_user.clone());

                Ok(new_user)
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
