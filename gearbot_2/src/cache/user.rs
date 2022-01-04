use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tracing::trace;
use twilight_model::id::{GuildId, UserId};
use twilight_model::user::{UserFlags, User as TwilightUser};
use crate::Cache;
use crate::cache::Guild;

pub struct User {
    pub name: String,
    pub discriminator: u16,
    pub avatar: Option<String>,
    pub bot: bool,
    //not caching system tag since these can't be members of a guild
    //todo: add banner once available to bots
    pub flags: UserFlags,

    // Users can only be on 200 guilds so u8 is plenty
    pub mutual_guilds: AtomicU8,
}

impl User {
    pub fn assemble(user: TwilightUser, old: Option<Arc<User>>) -> Self {
        let mutual_servers = old.map(|user| AtomicU8::new(user.mutual_guilds.load(Ordering::SeqCst))).unwrap_or_else(|| AtomicU8::new(0));

        User {
            name: user.name,
            discriminator: user.discriminator,
            avatar: user.avatar,
            bot: user.bot,
            flags: user.public_flags.unwrap_or_else(UserFlags::empty),
            mutual_guilds: mutual_servers
        }
    }

    // some properties can't change so only check those that could have been changed
    pub fn is_updated(&self, user: &TwilightUser) -> bool{
        self.name != user.name
        || self.discriminator != user.discriminator
        || self.avatar != user.avatar
        || self.flags != user.public_flags.unwrap_or_else(UserFlags::empty)
    }
}

impl Cache {
    pub fn get_user(&self, user_id: &UserId) -> Option<Arc<User>> {
        self.users.read().get(user_id).cloned()
    }

    // bulk insert users. doesn't return old values since this is for member chunk processing
    // thus this should only ever process new users
    pub fn insert_users(&self, users: Vec<(UserId, Arc<User>)>) {
        trace!("Inserting {} new users into the user cache", users.len());
        let mut cached_users = self.users.write();
        for (user_id, user) in users {
            cached_users.insert(user_id, user);
        }
    }

    pub fn insert_user(&self, user_id: UserId, user: Arc<User>) -> Option<Arc<User>> {
        self.users.write().insert(user_id, user)
    }

    pub fn cleanup_user(&self, user_id: &UserId) {
        self.users.write().remove(user_id);
    }

    pub fn for_each_guild(&self, mut todo: impl FnMut(&GuildId, &Arc<Guild>)) {
        for (guild_id, guild) in self.guilds.read().iter(){
            todo(guild_id, guild);
        }
    }
}