use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use twilight_model::id::{GuildId, UserId};

pub mod channel;
pub mod emoji;
pub mod guild;
pub mod member;
pub mod role;
pub mod user;
pub mod voice_state;

pub use channel::Channel;
pub use emoji::Emoji;
pub use guild::Guild;
pub use member::Member;
pub use role::Role;
pub use user::User;

pub struct Cache {
    guilds: RwLock<HashMap<GuildId, Arc<Guild>>>,
    unavailable_guilds: RwLock<Vec<GuildId>>,

    users: RwLock<HashMap<UserId, Arc<User>>>,
}

impl Cache {
    #[allow(clippy::new_without_default)]
    pub fn new_cache() -> Self {
        Cache {
            guilds: Default::default(),
            unavailable_guilds: Default::default(),
            users: Default::default(),
        }
    }
}
