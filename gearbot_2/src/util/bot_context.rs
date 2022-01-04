use std::collections::HashMap;
use std::ops::Range;
use std::sync::atomic::AtomicBool;
use parking_lot::RwLock;
use twilight_gateway::Cluster;
use twilight_http::Client;
use twilight_model::id::GuildId;
use gearbot_2_lib::translations::Translator;
use crate::cache::Cache;
use crate::Metrics;

pub struct BotContext {
    pub translator: Translator,
    pub client: Client,
    pub cluster: Cluster,
    pub metrics: Metrics,
    pub cache: Cache,

    requested_guilds: HashMap<u64, RwLock<Vec<GuildId>>>,
    pub pending_chunks: HashMap<u64, AtomicBool>
}

impl BotContext {
    pub fn new(translator: Translator, client: Client, cluster: Cluster, cluster_id: u16, shards: Range<u64>) -> Self {
        let mut requested_guilds = HashMap::new();
        let mut pending_chunks = HashMap::new();
        for shard_id in shards {
            requested_guilds.insert(shard_id, RwLock::new(Vec::new()));
            pending_chunks.insert(shard_id, AtomicBool::new(false));
        }

        BotContext {
            translator,
            client,
            cluster,
            metrics: Metrics::new(cluster_id),
            cache: Cache::new(),
            requested_guilds,
            pending_chunks
        }

    }

    pub fn has_requested_guilds(&self, shard: &u64) -> bool {
        !self.requested_guilds.get(shard).unwrap().read().is_empty()
    }

    pub fn get_next_requested_guild(&self, shard: &u64) -> Option<GuildId> {
        self.requested_guilds.get(shard).unwrap().write().pop()
    }

    pub fn add_requested_guild(&self, shard: &u64, guild_id: GuildId) {
        let mut requested = self.requested_guilds.get(shard).unwrap().write();
        if !requested.contains(&guild_id) {
            requested.push(guild_id);
        }
    }
}