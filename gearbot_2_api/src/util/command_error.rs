use twilight_http::Error as TwilightError;
use twilight_http::request::application::interaction::update_original_response::UpdateOriginalResponseError;
use gearbot_2_lib::kafka::sender::KafkaSenderError;
use gearbot_2_lib::translations::{GearBotLangKey, Translator};

pub enum CommandError {
    Twilight(TwilightError),
    TwilightUpdateOriginal(UpdateOriginalResponseError),
    KafkaSend(KafkaSenderError),
    MissingOption(String),
}

impl CommandError {
    // User errors are issues with user input and not logged as errors
    pub fn is_user_error(&self) -> bool {
        match self {
            self::CommandError::MissingOption(_) => true,
            _ => false
        }
    }

    //Error to show to the user
    pub fn get_user_error(&self, translator: &Translator, lang_code: &str) -> String {
        match self {
            self::CommandError::MissingOption(name) =>
                translator.translate(lang_code, GearBotLangKey::MissingRequiredOption)
                    .arg("name", name.to_string())
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
            CommandError::Twilight(e) => format!("Twilight error: {}", e),
            CommandError::TwilightUpdateOriginal(e) => format!("Twilight error when trying to update an original message for an interaction: {}", e),
            CommandError::KafkaSend(e) => format!("Failed to send kafka message: {}", e),

            // this isn't called for user errors
            _ => "SOMEONE FORGOT TO PROPERLY MAP THIS!".to_string()
        }
    }
}

impl From<TwilightError> for CommandError {
    fn from(e: TwilightError) -> Self {
        CommandError::Twilight(e)
    }
}

impl From<UpdateOriginalResponseError> for CommandError {
    fn from(e: UpdateOriginalResponseError) -> Self {
        CommandError::TwilightUpdateOriginal(e)
    }
}

impl From<KafkaSenderError> for CommandError {
    fn from(e: KafkaSenderError) -> Self {
        CommandError::KafkaSend(e)
    }
}