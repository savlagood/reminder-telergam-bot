mod database;
mod handlers;
mod models;
mod scheduler;
mod utils;

use anyhow::Result;
use std::{path::Path, sync::Arc};
use teloxide::{macros::BotCommands, prelude::*};

use crate::{
    database::Database,
    handlers::{handle_commands, handle_remind_cron, handle_remind_single, handle_unknown_message},
    scheduler::Scheduler,
};

const SQLITE_DB_FILENAME: &str = "data/reminders.db";

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
enum Command {
    #[command(description = "начать работу с ботом")]
    Start,
    #[command(description = "удалить все напоминания")]
    Drop,
}

#[tokio::main]
async fn main() -> Result<()> {
    std::fs::create_dir_all("log")?;
    std::fs::create_dir_all("config")?;
    std::fs::create_dir_all("data")?;

    if !Path::new("config/log4rs.yaml").exists() {
        return Err(anyhow::anyhow!(
            "Missing config/log4rs.yaml configuration file"
        ));
    }

    log4rs::init_file("config/log4rs.yaml", Default::default())?;

    let bot = Bot::from_env();
    let db = Arc::new(Database::new(SQLITE_DB_FILENAME)?);
    let scheduler = Arc::new(Scheduler::new().await?);

    let reminders = db.load_reminders().await?;
    for reminder in reminders {
        scheduler.add_reminder(reminder, bot.clone()).await?;
    }

    log::info!("Bot started successfully");

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_commands),
        )
        .branch(
            dptree::entry()
                .filter(|msg: Message| msg.text().is_some_and(|t| t.starts_with("/remindcron")))
                .endpoint(handle_remind_cron),
        )
        .branch(
            dptree::entry()
                .filter(|msg: Message| msg.text().is_some_and(|t| t.starts_with("/remindsingle")))
                .endpoint(handle_remind_single),
        )
        .branch(dptree::entry().endpoint(handle_unknown_message));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db, scheduler])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
