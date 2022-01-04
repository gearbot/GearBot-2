use std::thread;
use std::error::Error;
use std::sync::Arc;
use actix_web::{App, HttpServer, middleware, rt, web};
use tracing::{info, trace};
use twilight_gateway::cluster::{ClusterBuilder, ShardScheme};
use twilight_model::gateway::Intents;
use twilight_model::gateway::payload::outgoing::update_presence::UpdatePresencePayload;
use twilight_model::gateway::presence::{Activity, ActivityType, Status};
use gearbot_2_lib::translations::Translator;
use gearbot_2_lib::util::get_twilight_client;
use crate::util::{BotContext, Metrics, serve_metrics};
use futures_util::StreamExt;
use twilight_model::gateway::event::Event;
use crate::cache::Cache;

mod util;
pub mod cache;
mod events;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    info!("GearBot 2 initializing!");

    //TODO: move this to a central management system
    let cluster_id = 0;
    let clusters = 1;
    let shards_per_cluster = 2;

    let client = get_twilight_client().await?;
    let translator = Translator::new("translations", "en_US".to_string());

    let intents = Intents::GUILDS | Intents::GUILD_MEMBERS | Intents::GUILD_BANS | Intents::GUILD_EMOJIS | Intents::GUILD_VOICE_STATES | Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES;
    let (cluster, mut events) = ClusterBuilder::new(client.token().unwrap(), intents)
        .shard_scheme(ShardScheme::try_from(((cluster_id * shards_per_cluster..(cluster_id + 1) * shards_per_cluster), shards_per_cluster * clusters))?)
        .presence(
            UpdatePresencePayload {
                activities: vec![
                    Activity {
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
                    }
                ],
                afk: false,
                since: None,
                status: Status::Online,
            }
        )
        .build()
        .await?;

    let context = Arc::new(BotContext::new(translator, client, cluster, cluster_id as u16, cluster_id * shards_per_cluster..(cluster_id + 1) * shards_per_cluster));

    let c = context.clone();
    // start webserver on different thread
    thread::spawn(move || {
        let sys = rt::System::new();

        // srv is server controller type, `dev::Server`
        let srv = HttpServer::new(move || {
            App::new()
                .app_data(c.clone())
                // enable logger
                .wrap(middleware::Logger::default())
                .route("/metrics", web::get().to(serve_metrics))
        })
            .bind("127.0.0.1:9090")?
            .workers(1) // this is just metrics, doesn't need to be able to handle much at all
            .run();

        sys.block_on(srv)
    });




    // start the cluster in the background
    let c = context.clone();
    tokio::spawn(async move {
        info!("Cluster connecting to discord...");
        c.clone().cluster.up().await;
        info!("All shards are up!")
    });


    while let Some((id, event)) = events.next().await {
        trace!("Shard: {}, Event: {:?}", id, event.kind());
        // update metrics first so we can move the event
        if let Some(name) = event.kind().name() {
            context.metrics.gateway_events.get_metric_with_label_values(&[&id.to_string(), name]).unwrap().inc();
        }

        // recalculate shard states if needed
        match event {
            Event::ShardConnected(_) |
            Event::ShardConnecting(_) |
            Event::ShardDisconnected(_) |
            Event::ShardIdentifying(_) |
            Event::ShardReconnecting(_) |
            Event::ShardResuming(_) => context.metrics.recalculate_shard_states(&context),
            _ => {}
        }
        // update cache
        events::handle_gateway_event(id, event, &context);




    }

    Ok(())
}
