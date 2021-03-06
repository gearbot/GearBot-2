use std::collections::HashMap;
use std::str::FromStr;

use once_cell::sync::OnceCell;
use twilight_model::id::EmojiId;

use crate::define_emoji;
use crate::error::EmojiError;
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::channel::ReactionType;

pub(super) const ANIMATED_EMOTE_KEY: &str = "a";

define_emoji!(
    Yes => "â",
    No => "đĢ",
    Info => "âšī¸",
    Warn => "â ī¸",
    Robot => "đ¤",
    Bug => "đ",
    Bad => "đļ",
    GearDiamond => "âī¸",
    GearGold => "âī¸",
    GearIron => "âī¸",
    GearStone => "âī¸",
    GearWood => "âī¸",
    Left => "âŦī¸",
    Right => "âĄī¸",
    Online => "đĸ",

    StaffBadge => "",
    PartnerBadge => "",
    HypesquadEvents => "",
    BraveryBadge => "",
    BrillianceBadge => "",
    BalanceBadge => "",
    BugHunterBadge => "",
    EarlySupporterBadge => "",
    BugHunterLvl2Badge => "",
    VerifiedBotDevBadge => ""
);

#[derive(Debug)]
pub struct EmojiOverride {
    pub for_chat: String,
    pub id: EmojiId,
    pub name: String,
}

pub struct EmojiInfo {
    pub animated: bool,
    pub name: String,
    pub id: u64,
}

pub static EMOJI_OVERRIDES: OnceCell<HashMap<String, EmojiOverride>> = OnceCell::new();

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! define_emoji {
    ($($name: ident => $fallback: literal), *) => {


        #[derive(Debug, Clone)]
        pub enum Emoji {
            $( $name ,)*
        }

        impl std::fmt::Display for Emoji {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl Emoji {

            pub fn get_fallback(&self)-> &'static str {
                match self {
                    $(Emoji::$name => $fallback ,)*
                }
            }

            pub fn for_chat(&self) -> &'static str {
                match EMOJI_OVERRIDES.get() {
                    Some(overrides) => match overrides.get(&self.to_string()) {
                        Some(thing) => &thing.for_chat,
                        None => self.get_fallback()
                    },
                    None => self.get_fallback()
                }
            }

            pub fn matches(&self, emoji: &ReactionType) -> bool {
                let o = match EMOJI_OVERRIDES.get() {
                    Some(overrides) => overrides.get(&self.to_string()),
                    None => None
                };
                match &emoji {
                    ReactionType::Custom { id, .. } => {
                        match o {
                            Some(o) => id.0 == o.id.0,
                            None => false
                        }
                    }
                    ReactionType::Unicode { name } => {
                        match o {
                            Some(_) => false,
                            None => *name == self.get_fallback()
                        }
                    }
                }
            }


            pub fn to_reaction(&self) -> RequestReactionType {
                let o = match EMOJI_OVERRIDES.get() {
                        Some(overrides) => overrides.get(&self.to_string()),
                        None => None
                    };
                if let Some(o) = o {
                    RequestReactionType::Custom{id: o.id, name: Some(o.name.clone())}
                } else {
                    RequestReactionType::Unicode{name: self.get_fallback().to_string()}
                }
            }

            pub fn url(&self) -> String {
                let o = match EMOJI_OVERRIDES.get() {
                        Some(overrides) => overrides.get(&self.to_string()),
                        None => None
                    };
                if let Some(o) = o {
                    format!("https://cdn.discordapp.com/emojis/{}.png", o.id)
                } else {
                    String::from("https://cdn.discordapp.com/emojis/529008659498270721.png")
                }
            }
        }

        impl FromStr for Emoji {
            type Err = EmojiError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                    $(stringify!($name) => Ok(Emoji::$name) ,)*
                    _ => Err(EmojiError::UnknownEmoji(s.to_string())),
                }
            }
        }
     };
    }
}
