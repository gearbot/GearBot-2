use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use actix_web::{middleware, rt, web, App, HttpServer};
use futures_util::StreamExt;
use git_version::git_version;
use tracing::{info, trace};
use twilight_gateway::cluster::{ClusterBuilder, ShardScheme};
use twilight_model::gateway::event::Event;
use twilight_model::gateway::payload::outgoing::update_presence::UpdatePresencePayload;
use twilight_model::gateway::presence::{Activity, ActivityType, Status};
use twilight_model::gateway::Intents;

use gearbot_2_lib::datastore::Datastore;
use gearbot_2_lib::translations::Translator;
use gearbot_2_lib::util::get_twilight_client;

use crate::cache::Cache;
use crate::events::on_ready;
use crate::util::bot_context::{BotContext, BotStatus};
use crate::util::{serve_metrics, Metrics};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_VERSION: &str = git_version!();

pub mod cache;
mod communication;
pub mod events;
pub mod util;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    info!("GearBot v{} ({}) initializing!", VERSION, GIT_VERSION);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("GearPool")
        .build()
        .expect("Failed to build tokio runtime");
    let result = runtime.block_on(async_main());

    if result.is_ok() {
        info!("GearBot main loop exited gracefully, giving the last tasks 30 seconds to finish cleaning up");
        runtime.shutdown_timeout(Duration::from_secs(30));
        info!("Shutdown complete!");
        return Ok(());
    }

    result
}

async fn async_main() -> Result<(), Box<dyn Error + Send + Sync>> {
    //TODO: move this to a central management system
    let cluster_id = 0;
    let clusters = 1;
    let shards_per_cluster = 1;

    let client = get_twilight_client().await?;
    let translator = Translator::new("translations", "en_US".to_string());

    let intents = Intents::GUILDS
        | Intents::GUILD_MEMBERS
        | Intents::GUILD_BANS
        | Intents::GUILD_EMOJIS
        | Intents::GUILD_VOICE_STATES
        | Intents::GUILD_MESSAGES
        | Intents::DIRECT_MESSAGES;
    let (cluster, mut events) = ClusterBuilder::new(client.token().unwrap(), intents)
        .shard_scheme(ShardScheme::try_from((
            (cluster_id * shards_per_cluster..(cluster_id + 1) * shards_per_cluster),
            shards_per_cluster * clusters,
        ))?)
        .presence(UpdatePresencePayload {
            activities: vec![Activity {
                application_id: None,
                assets: None,
                buttons: vec![],
                created_at: None,
                details: None,
                emoji: None,
                flags: None,
                id: None,
                instance: None,
                kind: ActivityType::Watching,
                name: "my shiny new gears turning".to_string(),
                party: None,
                secrets: None,
                state: None,
                timestamps: None,
                url: None,
            }],
            afk: false,
            since: None,
            status: Status::Online,
        })
        .build()
        .await?;

    let context = Arc::new(BotContext::new(
        translator,
        client,
        cluster,
        Datastore::initialize().await?,
        cluster_id as u16,
        cluster_id * shards_per_cluster..(cluster_id + 1) * shards_per_cluster,
        shards_per_cluster * clusters,
    ));

    // initialize kafka message listener whenever possible
    tokio::spawn(communication::initialize_when_lonely(context.clone()));

    let c = context.clone();
    // start webserver on different thread
    thread::spawn(move || {
        let c2 = c.clone();
        let sys = rt::System::new();

        // srv is server controller type, `dev::Server`
        let srv = HttpServer::new(move || {
            App::new()
                .app_data(c.clone())
                // enable logger
                .wrap(middleware::Logger::default())
                .route("/metrics", web::get().to(serve_metrics))
        })
        .bind("127.0.0.1:9091")?
        .workers(1) // this is just metrics, doesn't need to be able to handle much at all
        .run();

        let res = sys.block_on(srv);

        // this shuts down on sigterm (actix installing it's own signal handlers?), take the cluster down along with it
        if !c2.is_status(BotStatus::Terminating) {
            c2.shutdown();
        }

        res
    });

    // start the cluster in the background
    let c = context.clone();
    tokio::spawn(async move {
        info!("Cluster connecting to discord...");
        c.clone().cluster.up().await;
        info!("All shards are up!")
    });

    while let Some((id, event)) = events.next().await {
        if context.is_status(BotStatus::Terminating) {
            break;
        }

        trace!("Shard: {}, Event: {:?}", id, event.kind());
        // update metrics first so we can move the event
        if let Some(name) = event.kind().name() {
            context
                .metrics
                .gateway_events
                .get_metric_with_label_values(&[&id.to_string(), name])
                .unwrap()
                .inc();
        }

        // recalculate shard states if needed
        match &event {
            Event::ShardConnected(_)
            | Event::ShardConnecting(_)
            | Event::ShardDisconnected(_)
            | Event::ShardIdentifying(_)
            | Event::ShardReconnecting(_)
            | Event::ShardResuming(_) => context.metrics.recalculate_shard_states(&context),
            _ => {}
        }
        // update cache
        events::handle_gateway_event(id, event, &context);
    }

    info!("Bot event loop terminated, giving the final background tasks 30 seconds to finish up...");

    Ok(())
}
