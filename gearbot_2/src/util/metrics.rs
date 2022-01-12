use crate::BotContext;
use actix_web::{HttpRequest, HttpResponse, Responder};
use prometheus::{Encoder, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Metrics {
    pub registry: Registry,

    pub gateway_events: IntCounterVec,
    pub shard_states: IntGaugeVec,

    pub guilds: IntGaugeVec,

    pub members: IntGauge,
    pub users: IntGauge,

    pub status: IntGaugeVec,
}

impl Metrics {
    pub fn new(cluster: u16) -> Self {
        let mut labels = HashMap::new();
        labels.insert("cluster".to_string(), cluster.to_string());
        let registry = Registry::new_custom(Some("gearbot".to_string()), Some(labels)).unwrap();

        let gateway_events = IntCounterVec::new(
            Opts::new("gateway_events", "Received gateway events"),
            &["shard", "event"],
        )
        .unwrap();
        registry.register(Box::new(gateway_events.clone())).unwrap();

        let shard_states =
            IntGaugeVec::new(Opts::new("shard_states", "States of the shards"), &["shard", "state"]).unwrap();
        registry.register(Box::new(shard_states.clone())).unwrap();

        let guilds = IntGaugeVec::new(
            Opts::new("guilds", "Cached guilds per shard and cache state"),
            &["shard", "state"],
        )
        .unwrap();
        registry.register(Box::new(guilds.clone())).unwrap();

        let members = IntGauge::new("members", "Total cached members").unwrap();
        registry.register(Box::new(members.clone())).unwrap();

        let users = IntGauge::new("users", "Total cached members").unwrap();
        registry.register(Box::new(users.clone())).unwrap();

        let status = IntGaugeVec::new(Opts::new("status", "Cluster status"), &["status"]).unwrap();
        registry.register(Box::new(status.clone())).unwrap();

        Metrics {
            registry,
            gateway_events,
            shard_states,
            guilds,
            members,
            users,
            status,
        }
    }

    pub fn recalculate_shard_states(&self, state: &Arc<BotContext>) {
        self.shard_states.reset();
        for (shard_id, info) in state.cluster.info() {
            self.shard_states
                .get_metric_with_label_values(&[&shard_id.to_string(), &info.stage().to_string()])
                .unwrap()
                .inc();
        }
    }
}

pub async fn serve_metrics(request: HttpRequest) -> impl Responder {
    let context = request.app_data::<Arc<BotContext>>().unwrap();
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = context.metrics.registry.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponse::Ok().body(buffer)
}
