use anyhow::Result;
use std::error::Error;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::prelude::*;
use teloxide::types::ChatKind;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::types::InlineQueryResult;
use teloxide::types::InlineQueryResultArticle;
use teloxide::types::InputMessageContent;
use teloxide::types::InputMessageContentText;
use teloxide::types::Me;
use teloxide::types::MessageKind;
use teloxide::types::ParseMode;
use teloxide::types::UpdateKind;
use teloxide::types::User;
use teloxide::utils::command::BotCommands;
use teloxide::Bot;
use tracing::info;

pub struct SolanaBot {
    admin_id: i32,
    bot: Bot,
}

impl SolanaBot {
    pub fn new(token: String) -> Result<Self> {
        let bot = teloxide::Bot::new(token);

        Ok(Self { bot, admin_id: 0 })
    }

    pub async fn run(self) -> Result<()> {
        let SolanaBot { bot, admin_id } = self;

        let handler = dptree::entry()
            .branch(Update::filter_message().endpoint(message_handler))
            .branch(Update::filter_callback_query().endpoint(callback_handler))
            .branch(Update::filter_inline_query().endpoint(inline_query_handler));

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
    /// Display this text
    Help,
    /// Start
    Start,
    Buy,
    Sell,
    Seting,
    Lang,
}

pub async fn start(bot: Bot, id: ChatId) -> Result<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Help".to_string(), "help".to_string()),
        InlineKeyboardButton::callback("Ping".to_string(), "ping".to_string()),
        InlineKeyboardButton::callback("Buy".to_string(), "buy".to_string()),
        InlineKeyboardButton::callback("Start".to_string(), "start".to_string()),
        InlineKeyboardButton::callback("Sell".to_string(), "sell".to_string()),
    ]]);

    bot.send_message(id, "Let's start my frrend!".to_string())
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            Ok(Command::Start) => {
                // TODO: query default wallet
                info!("Who send msg: {}", me.user.id);

                // Create a list of buttons and send them.
                let keyboard = make_keyboard();
                bot.send_message(msg.chat.id, "Debian versions:")
                    .reply_markup(keyboard)
                    .await?;
            }
            Ok(Command::Buy) => {
                // Create a list of buttons and send them.
                let keyboard = make_keyboard();
                bot.send_message(msg.chat.id, "Debian versions:")
                    .reply_markup(keyboard)
                    .await?;
            }
            Ok(Command::Sell) => {
                // Create a list of buttons and send them.
                let keyboard = make_keyboard();
                bot.send_message(msg.chat.id, "Debian versions:")
                    .reply_markup(keyboard)
                    .await?;
            }
            Ok(Command::Seting) => {
                // Create a list of buttons and send them.
                let keyboard = make_keyboard();
                bot.send_message(msg.chat.id, "Debian versions:")
                    .reply_markup(keyboard)
                    .await?;
            }
            Ok(Command::Lang) => {
                // Create a list of buttons and send them.
                let keyboard = make_keyboard();
                bot.send_message(msg.chat.id, "Debian versions:")
                    .reply_markup(keyboard)
                    .await?;
            }
            Err(_) => {
                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}

async fn inline_query_handler(
    bot: Bot,
    q: InlineQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let choose_debian_version = InlineQueryResultArticle::new(
        "0",
        "Chose debian version",
        InputMessageContent::Text(InputMessageContentText::new("Debian versions:")),
    )
    .reply_markup(make_keyboard());

    bot.answer_inline_query(q.id, vec![choose_debian_version.into()])
        .await?;

    Ok(())
}

/// Creates a keyboard made by buttons in a big column.
fn make_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let root_keyboard = ["Buy", "Sell"];

    for key in root_keyboard.chunks(3) {
        let row = key
            .iter()
            .map(|&version| InlineKeyboardButton::callback(version.to_owned(), version.to_owned()))
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

/// When it receives a callback from a button it edits the message with all
/// those buttons writing a text with the selected Debian version.
///
/// **IMPORTANT**: do not send privacy-sensitive data this way!!!
/// Anyone can read data stored in the callback button.
async fn callback_handler(bot: Bot, q: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(version) = q.data {
        let text = format!("You chose: {version}");

        // Tell telegram that we've seen this query, to remove 🕑 icons from the
        // clients. You could also use `answer_callback_query`'s optional
        // parameters to tweak what happens on the client side.
        bot.answer_callback_query(q.id).await?;

        // Edit text of the message to which the buttons were attached
        if let Some(Message { id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }

        log::info!("You chose: {}", version);
    }

    Ok(())
}
