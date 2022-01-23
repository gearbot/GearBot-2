use std::env;
use std::sync::Arc;

use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use git_version::git_version;
use ring::signature;
use ring::signature::UnparsedPublicKey;
use tracing::{error, info};
use twilight_http::Client;
use twilight_model::id::GuildId;

use gearbot_2_lib::datastore::Datastore;
use gearbot_2_lib::kafka::sender::KafkaSender;
use gearbot_2_lib::translations::Translator;
use gearbot_2_lib::util::get_twilight_client;

use crate::middleware::{expose_metrics, PrometheusMetrics};
use crate::util::Metrics;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_VERSION: &str = git_version!();

mod interactions;
mod middleware;
pub mod util;

#[get("")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

pub struct State {
    pub public_key: UnparsedPublicKey<Vec<u8>>,
    pub discord_client: Client,
    pub translator: Translator,
    pub metrics: Metrics,
    pub kafka_sender: KafkaSender,
    pub datastore: Datastore,
    pub clusters: u64,
    pub shards_per_cluster: u64,
}

impl State {
    pub fn cluster_for_guild(&self, guild_id: &GuildId) -> u64 {
        let shard_id = (guild_id.get() >> 22) % (self.clusters * self.shards_per_cluster);
        shard_id / self.shards_per_cluster
    }

    pub fn queue_for_guild(&self, guild_id: &GuildId) -> String {
        format!("gearbot_cluster_{}", self.cluster_for_guild(guild_id))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    info!("GearBot v{} ({}) api initializing!", VERSION, GIT_VERSION);

    // reading env variables
    let hex_signature = env::var("PUBLIC_KEY").expect("Missing PUBLIC_KEY env variable!");
    let clusters = env::var("CLUSTERS")
        .expect("Missing CLUSTERS env variable!")
        .parse::<u64>()
        .expect("CLUSTERS was not a proper number");
    let shards_per_cluster = env::var("SHARDS_PER_CLUSTER")
        .expect("Missing SHARDS_PER_CLUSTER env variable!")
        .parse::<u64>()
        .expect("SHARDS_PER_CLUSTER was not a proper number");

    let client = get_twilight_client()
        .await
        .expect("Failed to construct twilight http client");

    let decoded_signature = hex::decode(hex_signature).expect("Failed to decode PUBLIC_KEY");
    let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, decoded_signature);

    //loading translations
    let translator = Translator::new("translations", "en_US".to_string());

    let datastore = match Datastore::initialize().await {
        Ok(datastore) => datastore,
        Err(e) => {
            error!("Failed to initialize the datastore: {:?}", e);
            return Ok(());
        }
    };

    // assemble shared state
    let inner_state = State {
        public_key,
        discord_client: client,
        translator,
        metrics: Metrics::new(),
        kafka_sender: KafkaSender::new(),
        datastore,
        clusters,
        shards_per_cluster,
    };
    let state = Arc::new(inner_state);

    HttpServer::new(move || {
        let root_path = env::var("API_PATH").unwrap_or_else(|_| "api".to_string());
        App::new()
            .wrap(Logger::default())
            .wrap(PrometheusMetrics::new(state.clone()))
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Version", env!("CARGO_PKG_VERSION")))
                    .add(("Content-Type", "application/json")),
            )
            .service(
                web::scope(&root_path)
                    .app_data(state.clone())
                    .service(hello)
                    .service(interactions::handle_interactions)
                    .service(expose_metrics),
            )
    })
    .keep_alive(90)
    .bind("0.0.0.0:4000")?
    .run()
    .await
}
