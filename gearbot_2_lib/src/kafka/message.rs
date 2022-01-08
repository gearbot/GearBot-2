use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum Message {
    General(General),
    Interaction {
        token: String,
        command: InteractionCommand
    }
}

impl Message {
    pub fn new_interaction(token: String, command: InteractionCommand) -> Self {
        Message::Interaction {
            token,
            command
        }
    }
}

#[derive(Encode, Decode, Debug)]
pub enum General {
    Placeholder
}

#[derive(Encode, Decode, Debug)]
pub enum InteractionCommand {
    Debug {
        component: String
    }
}