use std::sync::Arc;
use std::sync::atomic::Ordering;
use parking_lot::RwLock;
use twilight_model::datetime::Timestamp;
use twilight_model::gateway::payload::incoming::MemberUpdate;
use twilight_model::id::RoleId;
use twilight_model::guild::Member as TwilightMember;
use crate::cache::User;

pub struct Member {
    user: RwLock<Arc<User>>,
    pub nickname: Option<String>,
    avatar: Option<String>,
    pub roles: Vec<RoleId>,
    pub joined_at: Timestamp, // TODO: does this work well enough now or does this need converting?
    pub pending: bool,
    pub communication_disabled_until: Option<Timestamp>,
}

impl Member {
    pub fn convert_with_user(member: TwilightMember, old_user: Option<Arc<User>>) -> Member {
        Member {
            user: RwLock::new(Arc::new(User::assemble(member.user, old_user))),
            nickname: member.nick,
            avatar: member.avatar,
            roles: member.roles,
            joined_at: member.joined_at,
            pending: member.pending,
            communication_disabled_until: member.communication_disabled_until,
        }
    }

    pub fn convert_update(member: MemberUpdate, old_user: Option<Arc<User>>) -> Member {
        Member {
            user: RwLock::new(Arc::new(User::assemble(member.user, old_user))),
            nickname: member.nick,
            avatar: member.avatar,
            roles: member.roles,
            joined_at: member.joined_at,
            pending: member.pending,
            communication_disabled_until: member.communication_disabled_until,
        }
    }

    pub fn assemble(member: TwilightMember, user: Arc<User>) -> Self {
        Member {
            user: RwLock::new(user),
            nickname: member.nick,
            avatar: member.avatar,
            roles: member.roles,
            joined_at: member.joined_at,
            pending: member.pending,
            communication_disabled_until: member.communication_disabled_until,
        }
    }

    pub fn from_update(member: MemberUpdate, user: Arc<User>) -> Self {
        Member {
            user: RwLock::new(user),
            nickname: member.nick,
            avatar: member.avatar,
            roles: member.roles,
            joined_at: member.joined_at,
            pending: member.pending,
            communication_disabled_until: member.communication_disabled_until,
        }
    }

    pub fn is_updated(&self, member: &MemberUpdate) -> bool {
        self.nickname != member.nick
            || self.avatar != member.avatar
            || self.pending != member.pending
             || self.communication_disabled_until != member.communication_disabled_until
            || self.roles.len() != member.roles.len()
            || self.roles != member.roles
    }

    pub fn user(&self) -> Arc<User> {
        self.user.read().clone()
    }

    pub fn set_user(&self, user: Arc<User>) {
        *self.user.write() = user
    }

    //helper to avoid cloning a user arc just to modify the mutual guilds count
    pub fn add_mutual_guild(&self) -> u8 {
        self.user.read().mutual_guilds.fetch_add(1, Ordering::SeqCst)
    }

    //helper to avoid cloning a user arc just to modify the mutual guilds count
    pub fn remove_mutual_guild(&self) -> u8 {
        self.user.read().mutual_guilds.fetch_sub(1, Ordering::SeqCst)
    }

    pub fn get_mutual_guilds(&self) -> u8 {
        self.user.read().mutual_guilds.load(Ordering::SeqCst)
    }
}