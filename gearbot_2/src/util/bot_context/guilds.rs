use std::sync::Arc;

use tracing::{error, info};
use twilight_http::error::ErrorType;

use gearbot_2_lib::datastore::guild::GuildInfo;
use gearbot_2_lib::datastore::DatastoreResult;
use gearbot_2_lib::util::markers::{GuildId, UserId};
use gearbot_2_lib::util::GearResult;

use crate::cache::guild::GuildCacheState;
use crate::cache::Member;
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

    /// Gets a member from a guild, tries from the cache first with http fallback in case we have
    /// not gotten to caching this guild
    pub async fn get_guild_member(&self, guild_id: &GuildId, user_id: &UserId) -> GearResult<Option<Arc<Member>>> {
        // do we know about the guild?
        if let Some(guild) = self.cache.get_guild(guild_id) {
            // with the member pre-cached?
            return if let Some(member) = guild.get_member(user_id) {
                Ok(Some(member))
            } else {
                // no? is the guild fully cached?
                if guild.is_cache_state(GuildCacheState::Cached) {
                    Ok(None)
                } else {
                    // incomplete cache, fall back to asking the api
                    match self.api_client.guild_member(*guild_id, *user_id).exec().await {
                        // member received from the api
                        Ok(response) => {
                            // get the user modal and convert to a cache member
                            let member = response.model().await?;
                            let uid = member.user.id;
                            // convert member and user as needed for insertion
                            let member = if let Some(user) = self.cache.get_user(&member.user.id) {
                                Member::assemble(member, user)
                            } else {
                                let member = Member::convert_with_user(member, None);
                                self.cache.insert_user(uid, member.user());
                                self.metrics.users.inc();
                                member
                            };

                            let member = Arc::new(member);

                            guild.insert_member(uid, member.clone());
                            self.metrics.members.inc();

                            // all done, return the member
                            Ok(Some(member))
                        }
                        // something else went wrong
                        Err(e) => {
                            // was this an 404 not found cause the user simply isn't on the server?
                            if matches!(e.kind(), ErrorType::Response { status, .. } if *status == 404) {
                                return Ok(None);
                            }
                            //something else went wrong, raise it
                            Err(e.into())
                        }
                    }
                }
            };
        }

        Ok(None)
    }
}
