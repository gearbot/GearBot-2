use std::future::Future;
use std::sync::Arc;
use tracing::error;

use gearbot_2_lib::util::GearResult;
use twilight_model::gateway::event::Event;

pub use other::on_ready;

use crate::events::channel::{on_channel_create, on_channel_delete, on_channel_update};
use crate::events::emoji::on_emoji_update;
use crate::events::guild::{on_guild_create, on_guild_delete, on_guild_update, on_member_chunk};
use crate::events::member::{on_member_add, on_member_remove, on_member_update};
use crate::events::message::{on_message, on_message_update};
use crate::events::other::on_resume;
use crate::events::role::{on_role_create, on_role_delete, on_role_update};
use crate::events::thread::{
    on_thread_create, on_thread_delete, on_thread_members_update, on_thread_sync, on_thread_update,
};
use crate::events::voice::on_voice_state_update;
use crate::util::bot_context::BotContext;

mod channel;
mod emoji;
mod guild;
mod member;
mod message;
mod other;
mod role;
mod thread;
mod voice;

//Just a hub function to fan out to the relevant handlers
pub fn handle_gateway_event(shard: u64, event: Event, context: &Arc<BotContext>) {
    match event {
        Event::ChannelCreate(create) => on_channel_create(create.0, context),
        Event::ChannelDelete(delete) => on_channel_delete(delete.0, context),
        Event::ChannelUpdate(update) => on_channel_update(update.0, context),
        Event::GuildCreate(guild_create) => on_guild_create(shard, guild_create.0, context),
        Event::GuildDelete(guild_delete) => on_guild_delete(shard, *guild_delete, context),
        Event::GuildEmojisUpdate(emoji_update) => on_emoji_update(emoji_update, context),
        Event::GuildUpdate(guild_update) => on_guild_update(guild_update.0, context),
        Event::MemberAdd(member_add) => on_member_add(member_add.0, context),
        Event::MemberRemove(member_remove) => on_member_remove(member_remove, context),
        Event::MemberUpdate(member_update) => on_member_update(*member_update, context),
        Event::MemberChunk(chunk) => on_member_chunk(shard, chunk, context),
        Event::RoleCreate(role_create) => on_role_create(role_create, context),
        Event::RoleDelete(role_delete) => on_role_delete(role_delete, context),
        Event::RoleUpdate(role_update) => on_role_update(role_update, context),
        Event::ThreadCreate(thread_create) => on_thread_create(thread_create.0, context),
        Event::ThreadDelete(thread_delete) => on_thread_delete(thread_delete, context),
        Event::ThreadListSync(thread_sync) => on_thread_sync(thread_sync, context),
        Event::ThreadMemberUpdate(_) => {} // not useful
        Event::ThreadMembersUpdate(thread_members_update) => on_thread_members_update(thread_members_update, context),
        Event::ThreadUpdate(thread_update) => on_thread_update(thread_update.0, context),
        Event::UserUpdate(_) => {} // only fires for the current user, not very useful.
        Event::VoiceStateUpdate(voice_update) => on_voice_state_update(voice_update.0, context),
        Event::Resumed => on_resume(shard, context),
        Event::Ready(ready) => on_ready(*ready, shard, context.clone()),

        // these ones don't do anything cache related and are async only so we might as well spawn them here
        Event::MessageCreate(message_create) => {
            async_wrapper(on_message(*message_create, context.clone()), "message_create")
        }
        Event::MessageUpdate(message_update) => {
            async_wrapper(on_message_update(*message_update, context.clone()), "message_update")
        }
        _ => {}
    }
}

fn async_wrapper(todo: impl Future<Output = GearResult<()>> + Send + 'static, name: &'static str) {
    tokio::spawn(async move {
        if let Err(e) = todo.await {
            if e.is_user_error() {
                error!(
                    "A user error instead of a system error got raised in the handler! {}",
                    name,
                )
            } else {
                error!("Failed to execute {} handler: {}", name, e.get_log_error())
            }
        }
    });
}
