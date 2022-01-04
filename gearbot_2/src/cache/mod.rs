use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use twilight_model::id::{GuildId, UserId};

pub mod guild;
pub mod role;
pub mod emoji;
pub mod channel;
pub mod member;
pub mod user;
pub mod voice_state;

pub use guild::Guild;
pub use role::Role;
pub use emoji::Emoji;
pub use channel::Channel;
pub use member::Member;
pub use user::User;

pub struct Cache {
    guilds: RwLock<HashMap<GuildId, Arc<Guild>>>,
    unavailable_guilds: RwLock<Vec<GuildId>>,

    users: RwLock<HashMap<UserId, Arc<User>>>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            guilds: Default::default(),
            unavailable_guilds: Default::default(),
            users: Default::default(),
        }
    }
}