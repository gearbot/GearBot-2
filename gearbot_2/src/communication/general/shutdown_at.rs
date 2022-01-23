use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tracing::info;

use crate::util::bot_context::{BotStatus, Context};

pub fn run(time: u128, uuid: u128, context: Context) {
    if context.uuid.as_u128() == uuid {
        info!("Received our own shutdown message, ignoring");
    } else {
        tokio::spawn(async move {
            let left = Duration::from_millis(
                time.saturating_sub(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()) as u64,
            );
            info!(
                "Shutdown command received from stand-by cluster, shutting down in {} seconds",
                left.as_secs_f32()
            );
            tokio::time::sleep(left).await;
            if !context.is_status(BotStatus::Terminating) {
                info!("Shutdown time reached!");
                context.shutdown();
            }
        });
    }
}
