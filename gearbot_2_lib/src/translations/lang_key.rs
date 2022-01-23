// Lang keys for everything GearBot can reply with.
// Using an enum to easily keep track of what is and isn't used.
// Logs should not be translated.
pub enum GearBotLangKey {
    //General
    UserId,

    //Time
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,

    //Userinfo command
    UserinfoUser,

    //Ping command
    PingCalculating,
    PingCalculated,

    //Debug localization string
    DebugLocalization,

    //Error replies
    GenericSystemError,
    MissingRequiredOption,
    InvalidOption,
    UnknownUser,
}

impl GearBotLangKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            GearBotLangKey::PingCalculating => "ping_calculating",
            GearBotLangKey::PingCalculated => "ping_calculated",
            GearBotLangKey::GenericSystemError => "generic_system_error",
            GearBotLangKey::MissingRequiredOption => "missing_required_option",
            GearBotLangKey::InvalidOption => "invalid_option",
            GearBotLangKey::DebugLocalization => "debug_localization",
            GearBotLangKey::UnknownUser => "unknown_user",
            GearBotLangKey::UserId => "user_id",
            GearBotLangKey::Years => "years",
            GearBotLangKey::Months => "months",
            GearBotLangKey::Weeks => "weeks",
            GearBotLangKey::Days => "days",
            GearBotLangKey::Hours => "hours",
            GearBotLangKey::Minutes => "minutes",
            GearBotLangKey::Seconds => "seconds",
            GearBotLangKey::UserinfoUser => "user_info_user",
        }
    }
}
