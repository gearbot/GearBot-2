use crate::cache::voice_state::VoiceState;
use crate::BotContext;
use std::sync::Arc;
use twilight_model::voice::VoiceState as TwilightVoiceState;

pub fn on_voice_state_update(update: TwilightVoiceState, context: &Arc<BotContext>) {
    if let Some(guild_id) = update.guild_id {
        if let Some(guild) = context.cache.get_guild(&guild_id) {
            let user_id = update.user_id;
            let new = VoiceState::from_state(update).map(|state| Arc::new(state));
            let old = guild.set_voice_state(user_id, new.clone());
        }
    }
}
