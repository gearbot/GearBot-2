// Lang keys for everything GearBot can reply with.
// Using an enum to easily keep track of what is and isn't used.
// Logs should not be translated.
pub enum GearBotLangKey {
    //Ping command
    PingCalculating,
    PingCalculated,
}

impl GearBotLangKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            GearBotLangKey::PingCalculating => "ping_calculating",
            GearBotLangKey::PingCalculated => "ping_calculated"
        }
    }
}