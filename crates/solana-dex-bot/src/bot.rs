use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::ChatKind;
use teloxide::types::ParseMode;
use teloxide::Bot;

use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::types::MessageKind;
use teloxide::types::UpdateKind;
use teloxide::types::User;
use teloxide::utils::command::BotCommands;

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

        teloxide::repl(bot, |bot: Bot, msg: Message| async move {
            dbg!(&msg);

            if let ChatKind::Private(ref member) = msg.chat.kind {
                //let user_name = member.first_name.unwrap_or("Unknow".to_string());
                //TODO: if is first time meet a user_id, we create an Private amd show address to
                //  the users

                //bot.send_message(msg.chat.id, format!("Welcome, {}!", user_name))
                //    .await?;
                if let Some(command) = msg.text() {
                    match command {
                        "/start" => start(bot, msg.chat.id)
                            .await
                            .expect("failed to return start comand"),
                        "/buy" => start(bot, msg.chat.id)
                            .await
                            .expect("failed to return start comand"),
                        "/sell" => start(bot, msg.chat.id)
                            .await
                            .expect("failed to return start comand"),
                        _ => {}
                    }
                }
            }

            Ok(())
        })
        .await;

        Ok(())
    }
}

pub async fn start(bot: Bot, id: ChatId) -> Result<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Help".to_string(), "/help".to_string()),
        InlineKeyboardButton::callback("Ping".to_string(), "/ping".to_string()),
        InlineKeyboardButton::callback("Buy".to_string(), "/buy".to_string()),
        InlineKeyboardButton::callback("Start".to_string(), "/start".to_string()),
        InlineKeyboardButton::callback("Sell".to_string(), "/sell".to_string()),
    ]]);

    bot.send_message(id, "Let's start my frrend!".to_string())
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
