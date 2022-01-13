use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct V1Config {
    pub moderation_logs: ModLog,
    pub message_logs: MessageLogs,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ModLog {
    pub style: LogStyle,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageLogs {
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LogStyle {
    Text,
    Embed,
}
