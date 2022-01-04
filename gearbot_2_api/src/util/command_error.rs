use twilight_http::Error as TwilightError;
use twilight_http::request::application::interaction::update_original_response::UpdateOriginalResponseError;

pub enum CommandError {
    Twilight(TwilightError),
    TwilightUpdateOriginal(UpdateOriginalResponseError)
}

impl CommandError {
    // User errors are issues with user input and not logged as errors
    pub fn is_user_error(&self) -> bool {
        match self {
            _ => false
        }
    }

    //Error to show to the user
    pub fn get_user_error(&self) -> &str {
        "Something went wrong!"
    }

    //Technical error for the logs
    pub fn get_log_error(&self) -> String {
        match self {
            CommandError::Twilight(e) => format!("Twilight error: {}", e),
            CommandError::TwilightUpdateOriginal(e) => format!("Twilight error when trying to update an original message for an interaction: {}", e)
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