use std::collections::HashMap;
use std::env;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::{OnceCell, RwLock as AsyncRwLock, SetError};
use tokio::task::JoinHandle;
use tracing::info;
use twilight_gateway::Cluster;
use twilight_http::client::InteractionClient;
use twilight_http::Client;
use uuid::Uuid;

use gearbot_2_lib::datastore::guild::GuildInfo;
use gearbot_2_lib::datastore::Datastore;
use gearbot_2_lib::translations::Translator;
use gearbot_2_lib::util::markers::{ApplicationId, GuildId};
pub use status::BotStatus;

use crate::cache::Cache;
use crate::util::bot_context::cluster_info::ClusterInfo;
use crate::Metrics;

mod cluster_info;
mod guilds;
mod status;
mod user;

pub type Context = Arc<BotContext>;

pub struct BotContext {
    pub translator: Translator,
    pub api_client: Client,
    pub bot_id: ApplicationId,
    pub cluster: Cluster,
    pub metrics: Metrics,
    pub cache: Cache,
    pub datastore: Datastore,

    status: RwLock<BotStatus>,
    pub cluster_info: ClusterInfo,

    requested_guilds: HashMap<u64, RwLock<Vec<GuildId>>>,
    pub pending_chunks: HashMap<u64, AtomicBool>,

    //uuid used to identify this instance
    pub uuid: Uuid,
    // the kafka receiver task once started
    receiver_handle: OnceCell<JoinHandle<()>>,

    /// Config cache
    cached_guild_info: AsyncRwLock<HashMap<GuildId, Arc<GuildInfo>>>,
}

impl BotContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        translator: Translator,
        client: Client,
        cluster: Cluster,
        datastore: Datastore,
        cluster_id: u16,
        shards: Range<u64>,
        total_shards: u64,
        bot_id: ApplicationId,
    ) -> Self {
        let mut requested_guilds = HashMap::new();
        let mut pending_chunks = HashMap::new();
        for shard_id in shards.clone() {
            requested_guilds.insert(shard_id, RwLock::new(Vec::new()));
            pending_chunks.insert(shard_id, AtomicBool::new(false));
        }

        let metrics = Metrics::new(cluster_id);
        metrics
            .status
            .get_metric_with_label_values(&[BotStatus::Starting.name()])
            .unwrap()
            .set(1);
        BotContext {
            translator,
            api_client: client,
            bot_id,
            cluster,
            metrics,
            cache: Cache::new_cache(),
            requested_guilds,
            pending_chunks,
            status: RwLock::new(BotStatus::Starting),
            cluster_info: ClusterInfo {
                cluster_id,
                shards,
                cluster_identifier: env::var("CLUSTER_IDENTIFIER").unwrap_or_else(|_| "gearbot".to_string()),
                total_shards,
            },
            uuid: Uuid::new_v4(),
            receiver_handle: Default::default(),
            datastore,
            cached_guild_info: Default::default(),
        }
    }

    pub fn has_requested_guilds(&self, shard: &u64) -> bool {
        !self.requested_guilds.get(shard).unwrap().read().is_empty()
    }

    pub fn has_any_requested_guilds(&self) -> bool {
        for (shard, pending) in self.requested_guilds.iter() {
            if !pending.read().is_empty() || self.pending_chunks.get(shard).unwrap().load(Ordering::SeqCst) {
                return true;
            }
        }
        false
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

    pub fn add_requested_guilds(&self, shard: &u64, guild_ids: Vec<GuildId>) {
        let mut requested = self.requested_guilds.get(shard).unwrap().write();
        for guild_id in guild_ids {
            if !requested.contains(&guild_id) {
                requested.push(guild_id);
            }
        }
    }

    pub fn clear_requested_guilds(&self) {
        for list in self.requested_guilds.values() {
            list.write().clear();
        }
    }

    pub fn shutdown(&self) {
        info!("Shutdown initiated...");
        self.set_status(BotStatus::Terminating);
        self.cluster.down();

        if let Some(handle) = self.receiver_handle.get() {
            info!("Handle found, killing queue listener");
            handle.abort();
        }
    }

    pub fn get_queue_topic(&self) -> String {
        format!(
            "{}_cluster_{}",
            self.cluster_info.cluster_identifier, self.cluster_info.cluster_id
        )
    }

    pub fn set_receiver_handle(&self, handle: JoinHandle<()>) -> Result<(), SetError<JoinHandle<()>>> {
        self.receiver_handle.set(handle)
    }

    pub fn interaction_client(&self) -> InteractionClient {
        self.api_client.interaction(self.bot_id)
    }
}
