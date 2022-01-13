use twilight_embed_builder::EmbedError;
use twilight_http::Error;
use gearbot_2_lib::datastore::{DatastoreError, DatastoreResult};
use gearbot_2_lib::translations::{GearBotLangKey, Translator};
use crate::util::error::InteractionError::Datastore;

pub enum InteractionError {
    InvalidOption(String),
    Twilight(twilight_http::Error),
    Embed(EmbedError),
    Datastore(DatastoreError),
    Serde(serde_json::Error),
}

impl InteractionError {
    pub fn is_user_error(&self) -> bool {
        match self {
            InteractionError::Twilight(_) => false,
            InteractionError::Embed(_) => false,
            InteractionError::Datastore(_) => false,
            InteractionError::Serde(_) => false,
            _ => true
        }
    }

    //Error to show to the user
    pub fn get_user_error(&self, translator: &Translator, lang_code: &str) -> String {
        match self {
            InteractionError::InvalidOption(choice) =>
                translator.translate(lang_code, GearBotLangKey::InvalidOption)
                    .arg("input", choice.to_string())
                    .build()
                    .to_string(),

            // Default generic error for system issues
            _ =>
                translator.translate(lang_code, GearBotLangKey::GenericSystemError).build().to_string()
        }
    }

    //Technical error for the logs
    pub fn get_log_error(&self) -> String {
        match self {
            InteractionError::Twilight(e) => format!("Twilight error: {}", e),
            InteractionError::Embed(e) => format!("Error assembling an embed: {}", e),
            InteractionError::Datastore(e) => format!("Datastore error: {}", e),
            InteractionError::Serde(e) => format!("Serde error: {}", e),
            // this isn't called for user errors
            _ => "SOMEONE FORGOT TO PROPERLY MAP THIS!".to_string()
        }
    }
}


impl From<twilight_http::Error> for InteractionError {
    fn from(e: Error) -> Self {
        InteractionError::Twilight(e)
    }
}

impl From<EmbedError> for InteractionError {
    fn from(e: EmbedError) -> Self {
        InteractionError::Embed(e)
    }
}

impl From<DatastoreError> for InteractionError {
    fn from(e: DatastoreError) -> Self {
        InteractionError::Datastore(e)
    }
}

impl From<serde_json::Error> for InteractionError {
    fn from(e: serde_json::Error) -> Self {
        InteractionError::Serde(e)
    }
}