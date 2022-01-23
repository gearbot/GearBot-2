use twilight_embed_builder::image_source::ImageSourceUrlError;
use twilight_embed_builder::EmbedError;
use twilight_http::response::DeserializeBodyError;
use twilight_http::Error;
use twilight_validate::message::MessageValidationError;

use crate::datastore::DatastoreError;
use crate::kafka::sender::KafkaSenderError;
use crate::translations::{GearBotLangKey, Translator};
use crate::util::markers::UserId;

pub enum GearError {
    //User errors
    InvalidOption(String),
    MissingOption(String),
    UnknownUser(UserId),

    //System errors
    Twilight(twilight_http::Error),
    Embed(EmbedError),
    Datastore(DatastoreError),
    Serde(serde_json::Error),
    KafkaSend(KafkaSenderError),
    DeserializeBody(DeserializeBodyError),
    SourceImageUrl(ImageSourceUrlError),
    MessageValidation(MessageValidationError),
}

impl GearError {
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            GearError::InvalidOption(_) | GearError::MissingOption(_) | GearError::UnknownUser(_)
        )
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

            GearError::UnknownUser(id) => translator
                .translate(lang_code, GearBotLangKey::UnknownUser)
                .arg("userid", id.to_string())
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
            GearError::KafkaSend(e) => format!("Failed to send kafka message: {}", e),
            GearError::DeserializeBody(e) => format!("Failed to deserialize the api response body: {:?}", e),
            GearError::SourceImageUrl(e) => format!("Invalid source url in an embed: {}", e),
            GearError::MessageValidation(e) => format!("Failed to assemble a proper message to send: {}", e),
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

impl From<KafkaSenderError> for GearError {
    fn from(e: KafkaSenderError) -> Self {
        GearError::KafkaSend(e)
    }
}

impl From<DeserializeBodyError> for GearError {
    fn from(e: DeserializeBodyError) -> Self {
        GearError::DeserializeBody(e)
    }
}

impl From<ImageSourceUrlError> for GearError {
    fn from(e: ImageSourceUrlError) -> Self {
        GearError::SourceImageUrl(e)
    }
}

impl From<MessageValidationError> for GearError {
    fn from(e: MessageValidationError) -> Self {
        GearError::MessageValidation(e)
    }
}
