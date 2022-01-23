use std::ops::Deref;

pub use config::DatabaseGuildInfo;
pub use config::GuildConfig;
pub use config::GuildConfigWrapper;
pub use config::GuildInfo;
pub use config::CURRENT_CONFIG_VERSION;

use crate::datastore::crypto::EncryptionKey;
use crate::datastore::Datastore;
use crate::util::markers::GuildId;

mod config;
mod message;

pub struct GuildDatastore<'a> {
    master_datastore: &'a Datastore,
    encryption_key: &'a EncryptionKey<'a>,
    guild_id: i64,
}

impl<'a> GuildDatastore<'a> {
    pub fn new(master_datastore: &'a Datastore, encryption_key: &'a EncryptionKey<'a>, guild_id: &'a GuildId) -> Self {
        GuildDatastore {
            master_datastore,
            encryption_key,
            guild_id: guild_id.get() as i64,
        }
    }
}

// if we deref to the master we can easily use those methods without bouncers
impl<'a> Deref for GuildDatastore<'a> {
    type Target = &'a Datastore;

    fn deref(&self) -> &Self::Target {
        &self.master_datastore
    }
}
