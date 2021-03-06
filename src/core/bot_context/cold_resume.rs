use std::collections::HashMap;

use log::info;
use twilight_model::gateway::presence::{ActivityType, Status};

use super::BotContext;
use crate::core::ColdRebootData;
use crate::error::ColdResumeError;
use crate::gearbot_important;

impl BotContext {
    pub async fn initiate_cold_resume(&self) -> Result<(), ColdResumeError> {
        // preparing for update rollout, set status to atleast give some indication to users
        gearbot_important!("Preparing for cold resume!");
        let _ = self
            .set_cluster_activity(
                Status::Idle,
                ActivityType::Watching,
                String::from("the new update being deployed. Replies might be delayed a bit"),
            )
            .await;

        let start = std::time::Instant::now();

        //kill the shards and get their resume info
        //DANGER: WE WILL NOT BE GETTING EVENTS FROM THIS POINT ONWARDS, REBOOT REQUIRED

        info!("Resume data acquired");

        let redis_cache = &self.datastore.cache_pool;

        let resume_data = self.cluster.down_resumable();
        let (guild_chunks, user_chunks) = self.cache.prepare_cold_resume(&redis_cache).await;

        // prepare resume data
        let mut map = HashMap::with_capacity(resume_data.len());
        for (shard_id, data) in resume_data {
            map.insert(shard_id, (data.session_id, data.sequence));
        }

        let data = ColdRebootData {
            resume_data: map,
            total_shards: self.scheme_info.total_shards,
            guild_chunks,
            shard_count: self.scheme_info.shards_per_cluster,
            user_chunks,
        };

        redis_cache
            .set(
                &format!("cb_cluster_data_{}", self.scheme_info.cluster_id),
                &data,
                Some(180),
            )
            .await?;

        info!(
            "Cold resume preparations completed in {}ms!",
            start.elapsed().as_millis()
        );

        Ok(())
    }
}
