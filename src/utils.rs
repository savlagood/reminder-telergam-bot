use chrono::Utc;
use teloxide::types::Message;

pub fn format_log(msg: &Message, action: &str) -> String {
    let time = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let chat_id = msg.chat.id.0;
    let user_id = msg.from.as_ref().map_or(0, |user| user.id.0);
    let username = msg
        .from
        .as_ref()
        .and_then(|user| user.username.as_ref())
        .map_or("unknown".to_string(), |name| name.to_string());

    format!(
        "[{}] chat_id: {}, user_id: {}, username: {}, action: {}",
        time, chat_id, user_id, username, action
    )
}
