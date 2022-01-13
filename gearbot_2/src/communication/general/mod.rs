use std::sync::Arc;

use gearbot_2_lib::kafka::message::General;

use crate::util::bot_context::BotContext;

mod shutdown_at;

pub fn handle(message: General, context: Arc<BotContext>) {
    match message {
        General::Hello() => {}
        General::ShutdownAt { time, uuid } => shutdown_at::run(time, uuid, context),
    }
}
