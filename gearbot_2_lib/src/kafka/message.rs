use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum Message {
    General(General),
    Interaction {
        token: String,
        locale: String,
        command: InteractionCommand,
    },
}

impl Message {
    pub fn new_interaction(token: String, locale: String, command: InteractionCommand) -> Self {
        Message::Interaction { token, locale, command }
    }
}

#[derive(Encode, Decode, Debug)]
pub enum General {
    Hello(),
    ShutdownAt { time: u128, uuid: u128 },
}

#[derive(Encode, Decode, Debug)]
pub enum InteractionCommand {
    Debug { component: String, guild_id: u64 },
    Userinfo { user_id: u64, guild_id: u64 },
}
