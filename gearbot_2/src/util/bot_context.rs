use crate::cache::Cache;
use crate::Metrics;
use gearbot_2_lib::translations::Translator;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::env;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{OnceCell, SetError};
use tokio::task::JoinHandle;
use tracing::info;
use twilight_gateway::Cluster;
use twilight_http::Client;
use twilight_model::id::GuildId;
use uuid::Uuid;

#[derive(Clone, Eq, PartialEq)]
pub enum BotStatus {
    STARTING,
    STANDBY,
    PRIMARY,
    TERMINATING,
}

impl BotStatus {
    pub fn name(&self) -> &str {
        match self {
            BotStatus::STARTING => "STARTING",
            BotStatus::STANDBY => "STANDBY",
            BotStatus::PRIMARY => "PRIMARY",
            BotStatus::TERMINATING => "TERMINATING",
        }
    }
}

pub struct BotContext {
    pub translator: Translator,
    pub client: Client,
    pub cluster: Cluster,
    pub metrics: Metrics,
    pub cache: Cache,

    status: RwLock<BotStatus>,
    pub cluster_info: ClusterInfo,

    requested_guilds: HashMap<u64, RwLock<Vec<GuildId>>>,
    pub pending_chunks: HashMap<u64, AtomicBool>,

    //uuid used to identify this instance
    pub uuid: Uuid,
    // the kafka receiver task once started
    receiver_handle: OnceCell<JoinHandle<()>>,
}

pub struct ClusterInfo {
    pub cluster_id: u16,
    pub shards: Range<u64>,
    pub cluster_identifier: String,
    pub total_shards: u64,
}

impl BotContext {
    pub fn new(
        translator: Translator,
        client: Client,
        cluster: Cluster,
        cluster_id: u16,
        shards: Range<u64>,
        total_shards: u64,
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
            .get_metric_with_label_values(&[BotStatus::STARTING.name()])
            .unwrap()
            .set(1);
        BotContext {
            translator,
            client,
            cluster,
            metrics,
            cache: Cache::new(),
            requested_guilds,
            pending_chunks,
            status: RwLock::new(BotStatus::STARTING),
            cluster_info: ClusterInfo {
                cluster_id,
                shards,
                cluster_identifier: env::var("CLUSTER_IDENTIFIER").unwrap_or_else(|_| "gearbot".to_string()),
                total_shards,
            },
            uuid: Uuid::new_v4(),
            receiver_handle: Default::default(),
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
        self.set_status(BotStatus::TERMINATING);
        self.cluster.down();

        if let Some(handle) = self.receiver_handle.get() {
            info!("Handle found, killing queue listener");
            handle.abort();
        }
    }

    pub fn set_status(&self, new_status: BotStatus) {
        // get lock
        let mut status = self.status.write();

        info!("Cluster status change: {} => {}", status.name(), new_status.name());

        // update metrics
        self.metrics.status.reset();
        self.metrics
            .status
            .get_metric_with_label_values(&[new_status.name()])
            .unwrap()
            .set(1);

        //store new status
        *status = new_status;
    }

    pub fn is_status(&self, status: BotStatus) -> bool {
        *self.status.read() == status
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
}
