use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use cron::Schedule;
use log::{debug, info};
use regex::Regex;
use std::str::FromStr;
use std::sync::Arc;
use teloxide::prelude::*;

use crate::{database::Database, models::Reminder, scheduler::Scheduler, utils::format_log};

pub async fn handle_commands(
    bot: Bot,
    msg: Message,
    cmd: crate::Command,
    db: Arc<Database>,
    scheduler: Arc<Scheduler>,
) -> Result<()> {
    match cmd {
        crate::Command::Start => {
            info!("{}", format_log(&msg, "start command"));
            bot.send_message(
                msg.chat.id,
                "Привет! Я бот для напоминаний.\n\n\
                Используй команды:\n\
                /remindcron \"* * * * * *\" сообщение - для периодических напоминаний. Время в UTC\n\
                /remindsingle \"01.01.2024 10:00:00\" сообщение - для одноразовых. Время в UTC\n\
                /drop - удалить все напоминания",
            )
            .await?;
        }
        crate::Command::Drop => {
            info!("{}", format_log(&msg, "drop command"));
            let deleted = db.delete_chat_reminders(msg.chat.id.0).await?;
            bot.send_message(msg.chat.id, format!("Удалено напоминаний: {}", deleted))
                .await?;
            scheduler.reload(bot.clone(), db.clone()).await?;
        }
    }
    Ok(())
}

pub async fn handle_remind_cron(
    bot: Bot,
    msg: Message,
    db: Arc<Database>,
    scheduler: Arc<Scheduler>,
) -> Result<()> {
    let text = msg.text().unwrap_or_default();
    let re = Regex::new(r#"/remindcron\s+"([^"]+)"\s+(.+)"#)?;

    if let Some(caps) = re.captures(text) {
        let cron_pattern = caps.get(1).unwrap().as_str();
        let message = caps.get(2).unwrap().as_str();

        info!(
            "{}",
            format_log(&msg, &format!("new cron reminder: {}", message))
        );

        if let Err(e) = Schedule::from_str(cron_pattern) {
            bot.send_message(msg.chat.id, format!("Ошибка в cron-паттерне: {}", e))
                .await?;
            return Ok(());
        }

        let reminder =
            Reminder::new_cron(msg.chat.id.0, message.to_string(), cron_pattern.to_string());

        scheduler
            .add_reminder(reminder.clone(), bot.clone())
            .await?;
        db.add_reminder(&reminder).await?;

        bot.send_message(msg.chat.id, "Периодическое напоминание установлено!")
            .await?;
    } else {
        info!("{}", format_log(&msg, "invalid cron reminder format"));
        bot.send_message(
            msg.chat.id,
            "Время в UTC. Формат команды: /remindcron \"* * * * *\" сообщение\n\
            Пример: /remindcron \"0 0 12 * * *\" Обед!",
        )
        .await?;
    }
    Ok(())
}

pub async fn handle_remind_single(
    bot: Bot,
    msg: Message,
    db: Arc<Database>,
    scheduler: Arc<Scheduler>,
) -> Result<()> {
    let text = msg.text().unwrap_or_default();
    let re = Regex::new(r#"/remindsingle\s+"([^"]+)"\s+(.+)"#)?;

    if let Some(caps) = re.captures(text) {
        let datetime_str = caps.get(1).unwrap().as_str();
        let message = caps.get(2).unwrap().as_str();

        info!(
            "{}",
            format_log(&msg, &format!("new single reminder: {}", message))
        );

        match NaiveDateTime::parse_from_str(datetime_str, "%d.%m.%Y %H:%M:%S") {
            Ok(naive_dt) => {
                let utc_dt = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);

                if utc_dt <= Utc::now() {
                    bot.send_message(msg.chat.id, "Укажите время в будущем!")
                        .await?;
                    return Ok(());
                }

                let reminder = Reminder::new_single(msg.chat.id.0, message.to_string(), utc_dt);

                scheduler
                    .add_reminder(reminder.clone(), bot.clone())
                    .await?;
                db.add_reminder(&reminder).await?;

                bot.send_message(msg.chat.id, "Одноразовое напоминание установлено!")
                    .await?;
            }
            Err(e) => {
                info!(
                    "{}",
                    format_log(&msg, &format!("invalid datetime format: {}", e))
                );
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Неправильный формат времени: {}\nИспользуйте DD.MM.YYYY HH:MM:SS",
                        e
                    ),
                )
                .await?;
            }
        }
    } else {
        info!("{}", format_log(&msg, "invalid single reminder format"));
        bot.send_message(
            msg.chat.id,
            "Формат команды: /remindsingle \"DD.MM.YYYY HH:MM:SS\" сообщение\n\
            Пример: /remindsingle \"31.12.2023 23:59:00\" С Новым Годом!",
        )
        .await?;
    }
    Ok(())
}

pub async fn handle_unknown_message(_bot: Bot, msg: Message) -> Result<()> {
    debug!("{}", format_log(&msg, "unknown command"));
    Ok(())
}
