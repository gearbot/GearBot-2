use crate::translations::{GearBotLangKey, Translator};
use crate::util::error::GearError;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::env;
use std::error::Error;
use tracing::{info, warn};
use twilight_http::client::ClientBuilder;
use twilight_http::Client;
use twilight_model::channel::message::AllowedMentions;
use twilight_util::snowflake::Snowflake;

pub mod error;
pub mod url;

pub type GearResult<T> = Result<T, GearError>;

pub async fn get_twilight_client() -> Result<Client, Box<dyn Error + Send + Sync>> {
    let token = env::var("BOT_TOKEN")?;
    let mut builder = ClientBuilder::new()
        .token(token)
        .default_allowed_mentions(AllowedMentions::builder().build());

    if let Ok(url) = env::var("PROXY_URL") {
        warn!(
            "Proxy url found, all requests are now routed through {} and the local rate limiter has been DISABLED!",
            &url
        );
        builder = builder.proxy(url, true).ratelimiter(None)
    }

    let client = builder.build();

    info!("Validating discord api token...");

    let user = client.current_user().exec().await?.model().await?;

    let bot = client.current_user_application().exec().await?.model().await?;

    info!(
        "Api credentials validated: {}#{} ({}) and application id {}",
        user.name,
        user.discriminator(),
        user.id,
        bot.id
    );

    client.set_application_id(bot.id);

    Ok(client)
}

pub fn snowflake_timestamp(snowflake: &dyn Snowflake) -> DateTime<Utc> {
    DateTime::from_utc(NaiveDateTime::from_timestamp(snowflake.timestamp() / 1000, 0), Utc)
}

pub fn formatted_snowflake_timestamp(snowflake: &dyn Snowflake) -> String {
    snowflake_timestamp(snowflake).format("%A %d %B %Y (%T)").to_string()
}

pub fn snowflake_age(snowflake: &dyn Snowflake, max_parts: usize, lang: &str, translator: &Translator) -> String {
    translated_age(snowflake_timestamp(snowflake), max_parts, lang, translator)
}

pub fn translated_age(old: DateTime<Utc>, max_parts: usize, lang: &str, translator: &Translator) -> String {
    let mut seconds = Utc::now().signed_duration_since(old).num_seconds();
    let mut parts = Vec::new();

    let years = (seconds as f64 / (60.0 * 60.0 * 24.0 * 365.25)) as i64;
    if years > 0 {
        seconds -= (years as f64 * 60.0 * 60.0 * 24.0 * 365.25) as i64;
        parts.push(
            translator
                .translate(lang, GearBotLangKey::Years)
                .arg("count", years)
                .build()
                .to_string(),
        );
    }

    if parts.len() < max_parts {
        let months = seconds / (60 * 60 * 24 * 30);
        if months > 0 {
            seconds -= months * 60 * 60 * 24 * 30;
            parts.push(
                translator
                    .translate(lang, GearBotLangKey::Months)
                    .arg("count", months)
                    .build()
                    .to_string(),
            );
        }
    }

    if parts.len() < max_parts {
        let weeks = seconds / (60 * 60 * 24 * 7);
        if weeks > 0 {
            seconds -= weeks * 60 * 60 * 24 * 7;
            parts.push(
                translator
                    .translate(lang, GearBotLangKey::Weeks)
                    .arg("count", weeks)
                    .build()
                    .to_string(),
            );
        }
    }

    if parts.len() < max_parts {
        let days = seconds / (60 * 60 * 24);
        if days > 0 {
            seconds -= days * 60 * 60 * 24;
            parts.push(
                translator
                    .translate(lang, GearBotLangKey::Days)
                    .arg("count", days)
                    .build()
                    .to_string(),
            );
        }
    }

    if parts.len() < max_parts {
        let hours = seconds / (60 * 60);
        if hours > 0 {
            seconds -= hours * 60 * 60;
            parts.push(
                translator
                    .translate(lang, GearBotLangKey::Hours)
                    .arg("count", hours)
                    .build()
                    .to_string(),
            );
        }
    }

    if parts.len() < max_parts {
        let minutes = seconds / 60;
        if minutes > 0 {
            seconds -= minutes * 60;
            parts.push(
                translator
                    .translate(lang, GearBotLangKey::Minutes)
                    .arg("count", minutes)
                    .build()
                    .to_string(),
            );
        }
    }

    if parts.len() < max_parts {
        parts.push(
            translator
                .translate(lang, GearBotLangKey::Seconds)
                .arg("count", seconds)
                .build()
                .to_string(),
        );
    }

    parts.join(" ")
}
