use serde::{Serialize, Deserialize};
use serde_json::Value;
use sqlx::FromRow;

mod history;
mod guild_config;

pub use guild_config::GuildConfig;
pub use guild_config::GuildInfo;
use crate::datastore::crypto::EncryptionKey;
use crate::datastore::guild::guild_config::history::V1Config;

/// The highest config version this application knows about and supports
pub const CURRENT_CONFIG_VERSION: i32 = 2;

#[derive(FromRow)]
pub struct DatabaseGuildInfo {
    pub id: i64,
    pub version: i32,
    pub config: Value,
    pub encryption_key: Vec<u8>,
}

impl DatabaseGuildInfo {
    pub fn has_supported_config(&self) -> bool {
        self.version <= CURRENT_CONFIG_VERSION
    }

    pub fn into_config_and_key(self) -> Result<GuildInfo, serde_json::Error> {
        let wrapper: GuildConfigWrapper = serde_json::from_value(self.config)?;
        Ok(
            GuildInfo {
                config: wrapper.into_config(),
                encryption_key: EncryptionKey::construct_owned(&self.encryption_key)
            }
        )
    }
}

/// Master struct for the versioned configs
/// names must start be V followed by a number, postgres uses this to extract and store the version number
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum GuildConfigWrapper {
    V1(V1Config),
    V2(GuildConfig),
}

impl GuildConfigWrapper {
    pub fn into_config(self) -> GuildConfig {
        let mut current = self;
        loop {
            match current {
                GuildConfigWrapper::V2(config) => {
                    return config;
                }
                outdated => current = outdated.migrate()
            }
        }
    }

    fn migrate(self) -> Self {
        match self {
            GuildConfigWrapper::V1(inner) => GuildConfigWrapper::V2(inner.into()),
            _ => panic!("Tried to migrate a fully migrated config!")
        }
    }
}