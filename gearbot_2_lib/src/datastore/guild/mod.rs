use std::ops::Deref;
use crate::datastore::crypto::EncryptionKey;
use crate::datastore::Datastore;

mod guild_config;

pub use guild_config::GuildConfig;
pub use guild_config::GuildConfigWrapper;
pub use guild_config::DatabaseGuildInfo;
pub use guild_config::GuildInfo;
pub use guild_config::CURRENT_CONFIG_VERSION;

pub struct GuildDatastore<'a> {
    master_datastore: &'a Datastore,
    encryption_key: EncryptionKey<'a>,
}

impl <'a>GuildDatastore<'a> {
    pub fn new(master_datastore: &'a Datastore, encryption_key: EncryptionKey<'a>)->Self {
        GuildDatastore {
            master_datastore,
            encryption_key
        }
    }
}


// if we deref to the master we can easily use those methods without bouncers
impl <'a> Deref for GuildDatastore<'a> {
    type Target = &'a Datastore;

    fn deref(&self) -> &Self::Target {
        &self.master_datastore
    }
}
