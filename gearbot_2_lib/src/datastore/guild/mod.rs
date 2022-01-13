use std::ops::Deref;

pub use config::DatabaseGuildInfo;
pub use config::GuildConfig;
pub use config::GuildConfigWrapper;
pub use config::GuildInfo;
pub use config::CURRENT_CONFIG_VERSION;

use crate::datastore::crypto::EncryptionKey;
use crate::datastore::Datastore;

mod config;

pub struct GuildDatastore<'a> {
    master_datastore: &'a Datastore,
    encryption_key: EncryptionKey<'a>,
}

impl<'a> GuildDatastore<'a> {
    pub fn new(master_datastore: &'a Datastore, encryption_key: EncryptionKey<'a>) -> Self {
        GuildDatastore {
            master_datastore,
            encryption_key,
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
