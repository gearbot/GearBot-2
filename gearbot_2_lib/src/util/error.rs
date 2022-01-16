use twilight_embed_builder::EmbedError;
use twilight_http::request::application::interaction::update_original_response::UpdateOriginalResponseError;
use twilight_http::Error;

use crate::datastore::DatastoreError;
use crate::kafka::sender::KafkaSenderError;
use crate::translations::{GearBotLangKey, Translator};

pub enum GearError {
    InvalidOption(String),
    MissingOption(String),

    Twilight(twilight_http::Error),
    Embed(EmbedError),
    Datastore(DatastoreError),
    Serde(serde_json::Error),
    TwilightUpdateOriginal(UpdateOriginalResponseError),
    KafkaSend(KafkaSenderError),
}

impl GearError {
    pub fn is_user_error(&self) -> bool {
        matches!(self, GearError::InvalidOption(_) | GearError::MissingOption(_))
    }

    //Error to show to the user
    pub fn get_user_error(&self, translator: &Translator, lang_code: &str) -> String {
        match self {
            GearError::InvalidOption(choice) => translator
                .translate(lang_code, GearBotLangKey::InvalidOption)
                .arg("input", choice.to_string())
                .build()
                .to_string(),

            GearError::MissingOption(name) => translator
                .translate(lang_code, GearBotLangKey::MissingRequiredOption)
                .arg("name", name.to_string())
                .build()
                .to_string(),

            // Default generic error for system issues
            _ => translator
                .translate(lang_code, GearBotLangKey::GenericSystemError)
                .build()
                .to_string(),
        }
    }

    //Technical error for the logs
    pub fn get_log_error(&self) -> String {
        match self {
            GearError::Twilight(e) => format!("Twilight error: {}", e),
            GearError::Embed(e) => format!("Error assembling an embed: {}", e),
            GearError::Datastore(e) => format!("Datastore error: {}", e),
            GearError::Serde(e) => format!("Serde error: {}", e),
            GearError::TwilightUpdateOriginal(e) => format!(
                "Twilight error when trying to update an original message for an interaction: {}",
                e
            ),
            GearError::KafkaSend(e) => format!("Failed to send kafka message: {}", e),
            // this isn't called for user errors
            _ => "SOMEONE FORGOT TO PROPERLY MAP THIS!".to_string(),
        }
    }
}

impl From<twilight_http::Error> for GearError {
    fn from(e: Error) -> Self {
        GearError::Twilight(e)
    }
}

impl From<EmbedError> for GearError {
    fn from(e: EmbedError) -> Self {
        GearError::Embed(e)
    }
}

impl From<DatastoreError> for GearError {
    fn from(e: DatastoreError) -> Self {
        GearError::Datastore(e)
    }
}

impl From<serde_json::Error> for GearError {
    fn from(e: serde_json::Error) -> Self {
        GearError::Serde(e)
    }
}

impl From<UpdateOriginalResponseError> for GearError {
    fn from(e: UpdateOriginalResponseError) -> Self {
        GearError::TwilightUpdateOriginal(e)
    }
}

impl From<KafkaSenderError> for GearError {
    fn from(e: KafkaSenderError) -> Self {
        GearError::KafkaSend(e)
    }
}
