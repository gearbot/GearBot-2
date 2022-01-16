use crate::util::error::GearError;
use std::env;
use std::error::Error;
use tracing::{info, warn};
use twilight_http::client::ClientBuilder;
use twilight_http::Client;
use twilight_model::channel::message::AllowedMentions;

pub mod error;

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
