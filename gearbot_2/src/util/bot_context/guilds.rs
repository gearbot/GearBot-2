use std::collections::HashMap;
use std::sync::Arc;

use tracing::{error, info};
use twilight_model::id::GuildId;

use gearbot_2_lib::datastore::guild::GuildInfo;
use gearbot_2_lib::datastore::DatastoreResult;

use crate::util::bot_context::BotContext;

impl BotContext {
    /// pre loading guild configurations so we don't get a thundering heard of them doing
    /// individual requests all at once when we get hit with 50+ events per second
    pub async fn load_initial_guilds(&self, guilds: Vec<GuildId>) {
        info!("Bulk loading configuration for {} guilds...", guilds.len());

        let mut loaded = self.cached_guild_info.write().await;
        let result = {
            let to_load = guilds
                .into_iter()
                .filter(|id| !loaded.contains_key(id))
                .collect::<Vec<GuildId>>();
            if !to_load.is_empty() {
                self.datastore.get_guild_info_bulk(to_load).await
            } else {
                return;
            }
        };

        match result {
            Ok(configs) => {
                for (id, config) in configs {
                    loaded.insert(id, Arc::new(config));
                }
            }
            Err(e) => {
                error!("Failed to bulk get guild configs: {}", e)
            }
        }
    }

    /// gets the guild info (config + encryption key) for a guild. If none exists one will be
    /// made and persisted
    pub async fn get_guild_info(&self, guild_id: &GuildId) -> DatastoreResult<Arc<GuildInfo>> {
        // first try the cache
        // the option block here is so cause it otherwise keeps the lock into the else
        let option = { self.cached_guild_info.read().await.get(guild_id).cloned() };
        if let Some(info) = option {
            Ok(info)
        } else {
            let result = self.datastore.get_or_create_guild_info(guild_id).await;
            // we get a lock here again but this is fine since this should be extremely rare due to the caching
            let mut cache = self.cached_guild_info.write().await;
            // check the cache again in case we failed due to a parallel race
            if let Some(info) = cache.get(guild_id) {
                Ok(info.clone())
            } else {
                let info = Arc::new(result?);
                cache.insert(*guild_id, info.clone());
                Ok(info)
            }
        }
    }
}
