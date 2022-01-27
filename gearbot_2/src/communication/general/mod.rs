use gearbot_2_lib::kafka::message::General;

use crate::util::bot_context::Context;

mod shutdown_at;

pub fn handle(message: General, context: Context) {
    match message {
        General::Hello() => {}
        General::ShutdownAt { time, uuid } => shutdown_at::run(time, uuid, context),
    }
}
