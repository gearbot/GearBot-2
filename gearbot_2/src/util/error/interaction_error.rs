use twilight_embed_builder::EmbedError;
use twilight_http::Error;
use gearbot_2_lib::translations::{GearBotLangKey, Translator};

pub enum InteractionError {
    InvalidOption(String),
    Twilight(twilight_http::Error),
    Embed(EmbedError)
}

impl InteractionError {
    pub fn is_user_error(&self) -> bool {
        match self {
            InteractionError::Twilight(_) => false,
            InteractionError::Embed(_) => false,
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