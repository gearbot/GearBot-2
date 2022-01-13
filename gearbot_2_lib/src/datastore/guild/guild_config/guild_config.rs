use serde::{Serialize, Deserialize};
use crate::datastore::crypto::EncryptionKey;
use crate::datastore::guild::guild_config::history::{LogStyle, MessageLogs, ModLog, V1Config};
use crate::datastore::guild::GuildConfigWrapper;

pub struct GuildInfo {
    pub config: GuildConfig,
    pub encryption_key: EncryptionKey<'static>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GuildConfig {
    pub moderation_logs: ModLog,
    pub message_logs: MessageLogs,
    pub anti_spam: AntiSpam
}

impl From<V1Config> for GuildConfig {
    fn from(previous: V1Config) -> Self {
        GuildConfig {
            moderation_logs: previous.moderation_logs,
            message_logs: previous.message_logs,
            anti_spam: AntiSpam { enabled: false }
        }
    }
}

impl Default for GuildConfig {
    fn default() -> Self {
        GuildConfig {
            moderation_logs: ModLog { style: LogStyle::Text },
            message_logs: MessageLogs { enabled: false },
            anti_spam: AntiSpam { enabled: false }
        }
    }
}

impl GuildConfig {
    pub fn wrapped(self) -> GuildConfigWrapper{
        GuildConfigWrapper::V2(self)
    }
}



#[derive(Clone, Serialize, Deserialize)]
pub struct AntiSpam {
    pub enabled: bool,
}