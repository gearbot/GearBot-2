use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use std::sync::Arc;

use actix_utils::future::{ready, Ready};

use actix_web::{body::MessageBody, Error, HttpResponse, Responder, Result, get, HttpRequest};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use chrono::{DateTime, Utc};
use futures_util::ready;
use pin_project_lite::pin_project;
use prometheus::{Encoder, TextEncoder};
use crate::State;


pub struct PrometheusMetrics(Arc<State>);

impl PrometheusMetrics {
    pub fn new(state: Arc<State>) -> Self {
        PrometheusMetrics(state)
    }
}


impl<S, B> Transform<S, ServiceRequest> for PrometheusMetrics
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        B: MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = PrometheusMetricsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PrometheusMetricsMiddleware {
            service,
            state: self.0.clone()
        }))
    }
}

/// Logger middleware service.
pub struct PrometheusMetricsMiddleware<S> {
    service: S,
    state: Arc<State>
}

impl<S, B> Service<ServiceRequest> for PrometheusMetricsMiddleware<S>
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        B: MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LoggerResponse<S, B>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let state = self.state.clone();
        let path = req.path().to_string();
        let method = req.method().to_string();
        LoggerResponse {
            fut: self.service.call(req),
            time: Utc::now(),
            _phantom: PhantomData,
            state,
            path,
            method,
        }
    }
}

pin_project! {
    pub struct LoggerResponse<S, B>
    where
        B: MessageBody,
        S: Service<ServiceRequest>,
    {
        #[pin]
        fut: S::Future,
        time: DateTime<Utc>,
        _phantom: PhantomData<B>,
        state: Arc<State>,
        path: String,
        method: String
    }
}

impl<S, B> Future for LoggerResponse<S, B>
    where
        B: MessageBody,
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
{
    type Output = Result<ServiceResponse<B>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let res = match ready!(this.fut.poll(cx)) {
            Ok(res) => res,
            Err(e) => return Poll::Ready(Err(e)),
        };
        let duration = Utc::now() - *this.time;
        let observation = (duration.num_microseconds().unwrap_or(i64::MAX) as f64) / 1_000_000f64;
        this.state.metrics.http_requests_duration.get_metric_with_label_values(&[this.method.as_str(), this.path.as_str(), res.status().as_str()]).unwrap().observe(observation);
        this.state.metrics.http_requests_total.get_metric_with_label_values(&[this.method.as_str(), this.path.as_str(), res.status().as_str()]).unwrap().inc();

        Poll::Ready(Ok(res))
    }
}

#[get("/metrics")]
pub async fn expose_metrics(request: HttpRequest) -> impl Responder {
    let state = request.app_data::<Arc<State>>().unwrap();
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = state.metrics.registry.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponse::Ok().body(buffer)
}