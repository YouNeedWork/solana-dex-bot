use anyhow::Result;
use std::error::Error;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;
use teloxide::Bot;
use tracing::info;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use std::sync::Arc;

type PGPool = Pool<ConnectionManager<PgConnection>>;

pub struct SolanaBot {
    admin_id: i32,
    bot: Bot,
    db: PGPool,
}

impl SolanaBot {
    pub fn new(token: String, db: PGPool) -> Result<Self> {
        let bot = teloxide::Bot::new(token);

        Ok(Self {
            bot,
            admin_id: 0,
            db,
        })
    }

    pub async fn run(self) -> Result<()> {
        let SolanaBot {
            bot,
            admin_id: _,
            db,
        } = self;

        let db = Arc::new(db);

        let handler = dptree::entry()
            .branch(
                Update::filter_message()
                    .endpoint(move |bot, msg, me| message_handler(bot, msg, me, db.clone())),
            )
            .branch(Update::filter_callback_query().endpoint(callback_handler));

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
    BuySell,
    Menu,
    Seting,
    Lang,
}

pub async fn menu(bot: Bot, id: ChatId) -> Result<()> {
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

    bot.send_message(id, "Let's start my frrend!".to_string())
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
    db: Arc<PGPool>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            Ok(Command::Start) => {
                //TODO if user not register then register and generate wallet for it
                menu(bot, msg.chat.id).await?;
            }
            Ok(Command::Menu) => {
                //  TODO: query default wallet
                // info!("Who send msg: {}", me.user.id);
                menu(bot, msg.chat.id).await?;
            }
            Ok(Command::BuySell) => {}
            Ok(Command::Seting) => {}
            Ok(Command::Lang) => {}
            Err(_) => {
                info!(text, "Got text");

                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}

async fn callback_handler(bot: Bot, q: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(chose) = q.data {
        log::info!("You chose: {}", chose);
        bot.answer_callback_query(q.id).await?;

        let text = if chose == "Buy" {
            "You chose to buy Debian!"
        } else {
            "You chose to sell Debian!"
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
