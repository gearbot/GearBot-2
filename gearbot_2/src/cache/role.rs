use std::sync::Arc;

use tracing::error;
use twilight_model::guild::{Permissions, Role as TwilightRole};
use twilight_model::util::ImageHash;

use gearbot_2_lib::util::markers::{GuildId, RoleId};

use crate::Cache;

pub struct Role {
    // cache role id as well since we need it for role ordering
    pub id: RoleId,
    pub name: String,
    pub color: u32,
    pub hoisted: bool,
    pub icon: Option<ImageHash>,
    pub emoji: Option<String>,
    pub position: i64,
    pub permissions: Permissions,
    pub managed: bool,
}

impl Role {
    pub fn from_role(role: TwilightRole) -> Self {
        Role {
            id: role.id,
            name: role.name,
            color: role.color,
            hoisted: role.hoist,
            icon: role.icon,
            emoji: role.unicode_emoji,
            position: role.position,
            permissions: role.permissions,
            managed: role.managed,
        }
    }
}

impl Cache {
    pub fn insert_role(&self, guild_id: &GuildId, role: Arc<Role>) -> Option<Arc<Role>> {
        if let Some(guild) = self.guilds.read().get(guild_id) {
            guild.insert_role(role)
        } else {
            error!("Tried to add a role to an uncached guild: {}", guild_id);
            None
        }
    }

    pub fn remove_role(&self, guild_id: &GuildId, role_id: &RoleId) -> Option<Arc<Role>> {
        if let Some(guild) = self.guilds.read().get(guild_id) {
            guild.remove_role(role_id)
        } else {
            error!("Tried to remove a role from an uncached guild: {}", guild_id);
            None
        }
    }
}
