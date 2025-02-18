use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Reminder {
    pub chat_id: i64,
    pub message: String,
    pub cron_pattern: Option<String>,
    pub due_time: Option<DateTime<Utc>>,
}

impl Reminder {
    pub fn new_cron(chat_id: i64, message: String, cron_pattern: String) -> Self {
        Self {
            chat_id,
            message,
            cron_pattern: Some(cron_pattern),
            due_time: None,
        }
    }

    pub fn new_single(chat_id: i64, message: String, due_time: DateTime<Utc>) -> Self {
        Self {
            chat_id,
            message,
            cron_pattern: None,
            due_time: Some(due_time),
        }
    }
}
