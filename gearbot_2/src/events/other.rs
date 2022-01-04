use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::{error, warn};
use crate::BotContext;
use crate::events::guild::request_next_guild;

pub fn on_ready(shard: u64, context: &Arc<BotContext>) {
    // make sure we don't think we are waiting on chunks. This only gets fired on new connections
    // so there can't be pending chunks on their way. We will get fresh creates for all guilds
    // so that will kick off the actual requesting.
    if context.pending_chunks.get(&shard).unwrap().fetch_and(false, Ordering::SeqCst) {
        warn!("Shard {} got a ready event while guild chunks where still pending!", shard);
    }
}

pub fn on_resume(shard: u64, context: &Arc<BotContext>) {
    if !context.has_requested_guilds(&shard) {
        error!("We have guilds queued up for member requesting, but got interrupted by a disconnect, resuming chunk requests");
        tokio::spawn(request_next_guild(shard, context.clone()));
    }
}

