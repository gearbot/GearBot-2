use twilight_model::guild::Emoji as TwilightEmoji;

use gearbot_2_lib::util::markers::RoleId;

pub struct Emoji {
    pub name: String,
    pub roles: Vec<RoleId>,
    pub animated: bool,
    pub available: bool,
}

impl Emoji {
    pub fn from_emoji(emoji: TwilightEmoji) -> Self {
        Emoji {
            name: emoji.name,
            roles: emoji.roles,
            animated: emoji.animated,
            available: emoji.available,
        }
    }
}
