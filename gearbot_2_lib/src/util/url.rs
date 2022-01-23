use twilight_model::id::{GuildId, UserId};

pub fn assemble_guild_avatar_url(guild_id: &GuildId, user_id: &UserId, avatar: &str) -> String {
    format!(
        "https://cdn.discordapp.com/guilds/{}/users/{}/avatars/{}.png",
        guild_id, user_id, avatar
    )
}

pub fn assemble_user_avatar(user_id: &UserId, discriminator: u16, avatar: Option<&String>) -> String {
    avatar.map_or_else(
        || format!("https://cdn.discordapp.com/embed/avatars/{}.png", discriminator % 5),
        |avatar| format!("https://cdn.discordapp.com/avatars/{}/{}.png", user_id, avatar),
    )
}
