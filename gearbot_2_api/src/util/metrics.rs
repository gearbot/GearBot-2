use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry};

pub struct Metrics {
    pub registry: Registry,

    // general http timings
    pub http_requests_total: IntCounterVec,
    pub http_requests_duration: HistogramVec,

    // interaction timings
    pub command_totals: IntCounterVec,
    pub command_durations: HistogramVec,

    // interaction async followups
    pub command_followups_total: IntCounterVec,
    pub command_followups_duration: HistogramVec,
}

impl Metrics {
    pub fn new() -> Self {
        // all of these are safe to unwrap as they only error when illegal arguments are passed, and none of these are dynamic
        let registry = Registry::new_custom(Some(String::from("gearbot")), None).unwrap();

        let http_requests_total = IntCounterVec::new(
            Opts::new(
                "http_requests_total",
                "Total handled http requests per method, route and status",
            ),
            &["method", "path", "status"],
        )
        .unwrap();
        registry.register(Box::new(http_requests_total.clone())).unwrap();

        let http_requests_duration = HistogramVec::new(
            HistogramOpts::new(
                "http_requests_duration",
                "Duration for the recent http requests per method, route and status",
            ),
            &["method", "path", "status"],
        )
        .unwrap();
        registry.register(Box::new(http_requests_duration.clone())).unwrap();

        let command_totals = IntCounterVec::new(
            Opts::new("command_totals", "Total handled command invocations by result"),
            &["command_name", "result"],
        )
        .unwrap();
        registry.register(Box::new(command_totals.clone())).unwrap();

        let command_durations = HistogramVec::new(
            HistogramOpts::new("command_durations", "Duration of initial command handler invocation")
                .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 1.5, 2.0, 3.0]),
            &["command", "result"],
        )
        .unwrap();
        registry.register(Box::new(command_durations.clone())).unwrap();

        let command_followups_total = IntCounterVec::new(
            Opts::new(
                "command_followups_total",
                "Total async command followups invocations by result",
            ),
            &["command_name", "result"],
        )
        .unwrap();
        registry.register(Box::new(command_followups_total.clone())).unwrap();

        let command_followups_duration = HistogramVec::new(
            HistogramOpts::new(
                "command_followups_duration",
                "Duration of async command followup handler invocations",
            )
            .buckets(vec![
                0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0,
            ]),
            &["command", "result"],
        )
        .unwrap();
        registry.register(Box::new(command_followups_duration.clone())).unwrap();

        Metrics {
            registry,
            http_requests_total,
            http_requests_duration,
            command_totals,
            command_durations,
            command_followups_total,
            command_followups_duration,
        }
    }
}
