use anyhow::Result;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::{database::Database, models::Reminder};

pub struct Scheduler {
    job_scheduler: Mutex<JobScheduler>,
}

impl Scheduler {
    pub async fn new() -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        scheduler.start().await?;
        Ok(Scheduler {
            job_scheduler: Mutex::new(scheduler),
        })
    }

    pub async fn add_reminder(&self, reminder: Reminder, bot: Bot) -> Result<()> {
        let chat_id = ChatId(reminder.chat_id);
        let message = reminder.message.clone();
        let scheduler = self.job_scheduler.lock().await;

        match (reminder.cron_pattern, reminder.due_time) {
            (Some(cron_pattern), _) => {
                let bot_clone = bot.clone();
                let job = Job::new_async(&cron_pattern, move |_uuid, _l| {
                    let bot = bot_clone.clone();
                    let msg = message.clone();
                    let chat_id = chat_id;
                    Box::pin(async move {
                        if let Err(e) = bot.send_message(chat_id, msg).await {
                            log::error!("Failed to send scheduled message: {}", e);
                        }
                    })
                })?;
                scheduler.add(job).await?;
            }
            (_, Some(due_time)) => {
                if due_time > chrono::Utc::now() {
                    let duration = due_time
                        .signed_duration_since(chrono::Utc::now())
                        .to_std()?;
                    let bot_clone = bot.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(duration).await;
                        if let Err(e) = bot_clone.send_message(chat_id, message).await {
                            log::error!("Failed to send scheduled message: {}", e);
                        }
                    });
                }
            }
            _ => log::warn!("Reminder without schedule or due time"),
        }

        Ok(())
    }

    pub async fn reload(&self, bot: Bot, db: Arc<Database>) -> Result<()> {
        let new_scheduler = JobScheduler::new().await?;
        new_scheduler.start().await?;

        {
            let mut lock = self.job_scheduler.lock().await;
            lock.shutdown().await?;
            *lock = new_scheduler;
        }

        let reminders = db.load_reminders().await?;
        for reminder in reminders {
            self.add_reminder(reminder, bot.clone()).await?;
        }

        Ok(())
    }
}
