use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    parse_with = "split",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Start use the bot")]
    Start,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "ping the bot.")]
    Ping,
}

pub async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .parse_mode(ParseMode::Markdown)
                .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .parse_mode(ParseMode::Markdown)
                .await?;
        }
        Command::Ping => {
            bot.send_message(msg.chat.id, "pong").await?;
        }
    };

    Ok(())
}
