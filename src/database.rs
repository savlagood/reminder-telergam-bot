use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
use crate::models::Reminder;

pub struct Database {
    connection: Mutex<Connection>,
}

impl Database {
    pub fn new(filename: &str) -> Result<Self> {
        let connection = Connection::open(filename)
            .with_context(|| format!("Failed to open database: {}", filename))?;

        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS reminders (
                id INTEGER PRIMARY KEY,
                chat_id INTEGER NOT NULL,
                message TEXT NOT NULL,
                cron_pattern TEXT,
                due_time DATETIME
            )",
                [],
            )
            .context("Failed to create reminders table")?;

        connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_reminders_chat_id ON reminders(chat_id)",
            [],
        )?;

        Ok(Database {
            connection: Mutex::new(connection),
        })
    }

    pub async fn add_reminder(&self, reminder: &Reminder) -> Result<()> {
        let conn = self.connection.lock().await;
        conn.execute(
            "INSERT INTO reminders (chat_id, message, cron_pattern, due_time)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                reminder.chat_id,
                reminder.message,
                reminder.cron_pattern,
                reminder.due_time.map(|dt| dt.to_rfc3339())
            ],
        )
        .context("Failed to add reminder to database")?;
        Ok(())
    }

    pub async fn delete_chat_reminders(&self, chat_id: i64) -> Result<usize> {
        let conn = self.connection.lock().await;
        let deleted = conn
            .execute("DELETE FROM reminders WHERE chat_id = ?1", params![chat_id])
            .context("Failed to delete reminders")?;
        Ok(deleted)
    }

    pub async fn load_reminders(&self) -> Result<Vec<Reminder>> {
        let conn = self.connection.lock().await;
        let mut stmt = conn
            .prepare("SELECT chat_id, message, cron_pattern, due_time FROM reminders")
            .context("Failed to prepare select statement")?;

        let reminders = stmt
            .query_map([], |row| {
                Ok(Reminder {
                    chat_id: row.get(0)?,
                    message: row.get(1)?,
                    cron_pattern: row.get(2)?,
                    due_time: row.get::<_, Option<String>>(3)?.map(|dt_str| {
                        DateTime::parse_from_rfc3339(&dt_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .expect("Invalid date format in database")
                    }),
                })
            })
            .context("Failed to query reminders")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect reminders")?;

        Ok(reminders)
    }
}
