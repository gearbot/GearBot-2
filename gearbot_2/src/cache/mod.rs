use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

pub use channel::Channel;
pub use emoji::Emoji;
use gearbot_2_lib::util::markers::{GuildId, UserId};
pub use guild::Guild;
pub use member::Member;
pub use role::Role;
pub use user::User;

pub mod channel;
pub mod emoji;
pub mod guild;
pub mod member;
pub mod role;
pub mod user;
pub mod voice_state;

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
