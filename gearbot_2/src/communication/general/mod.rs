use crate::BotContext;
use gearbot_2_lib::kafka::message::General;
use std::sync::Arc;

mod shutdown_at;

pub fn handle(message: General, context: Arc<BotContext>) {
    match message {
        General::Hello() => {}
        General::ShutdownAt { time, uuid } => shutdown_at::run(time, uuid, context),
    }
}
