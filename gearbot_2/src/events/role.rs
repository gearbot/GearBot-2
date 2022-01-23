use std::sync::Arc;
use twilight_model::gateway::payload::incoming::{RoleCreate, RoleDelete, RoleUpdate};

use crate::cache::Role;
use crate::util::bot_context::Context;

pub fn on_role_create(role_create: RoleCreate, context: &Context) {
    let new: Arc<Role> = Arc::new(Role::from_role(role_create.role));
    context.cache.insert_role(&role_create.guild_id, new);
}

pub fn on_role_update(role_update: RoleUpdate, context: &Context) {
    let new: Arc<Role> = Arc::new(Role::from_role(role_update.role));
    let _old = context.cache.insert_role(&role_update.guild_id, new);
}

pub fn on_role_delete(role_delete: RoleDelete, context: &Context) {
    let _old = context.cache.remove_role(&role_delete.guild_id, &role_delete.role_id);
}
