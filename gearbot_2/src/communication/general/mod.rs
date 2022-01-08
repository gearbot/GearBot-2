use std::sync::Arc;
use tracing::debug;
use gearbot_2_lib::kafka::message::General;
use crate::BotContext;

pub fn handle(message: General, context: Arc<BotContext>) {
    match message {
        General::Hello => {}
    }
}