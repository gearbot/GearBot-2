use std::sync::Arc;
use twilight_model::voice::VoiceState as TwilightVoiceState;

use crate::cache::voice_state::VoiceState;
use crate::util::bot_context::Context;

pub fn on_voice_state_update(update: TwilightVoiceState, context: &Context) {
    if let Some(guild_id) = update.guild_id {
        if let Some(guild) = context.cache.get_guild(&guild_id) {
            let user_id = update.user_id;
            let new = VoiceState::from_state(update).map(Arc::new);
            let _old = guild.set_voice_state(user_id, new);
        }
    }
}
