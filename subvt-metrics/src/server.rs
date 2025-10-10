use std::fmt::Debug;
use std::{convert::Infallible, net::ToSocketAddrs, string::FromUtf8Error};

use crate::registry::get_default_registry;
use prometheus::{Encoder, Registry, TextEncoder};
use tokio::{task, task::JoinError};
use warp::{reject::Reject, Filter, Rejection, Reply};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Failed to encode: {0}")]
    PrometheusEncodeFailed(#[from] prometheus::Error),
    #[error("Failed to encode: {0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("Failed to spawn blocking thread: {0}")]
    BlockingThreadFailed(#[from] JoinError),
}

pub async fn start<T: ToSocketAddrs + Debug>(address: T) {
    let registry = get_default_registry();
    let route = warp::path!("metrics")
        .and(with(registry))
        .and_then(metrics_text_handler);

    log::info!("Metrics server started on {address:?}");
    let routes = route.with(warp::log("subvt_metrics_server"));
    let socket_addr = address
        .to_socket_addrs()
        .expect("Invalid server address.")
        .next()
        .expect("Could not resolve address.");
    warp::serve(routes).run(socket_addr).await;
}

async fn metrics_text_handler(registry: Registry) -> Result<impl Reply, Rejection> {
    task::spawn_blocking::<_, Result<_, Error>>(move || {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&registry.gather(), &mut buffer)?;
        let encoded = String::from_utf8(buffer)?;
        Ok(encoded)
    })
    .await
    .map_err(Error::BlockingThreadFailed)?
    .map_err(Into::into)
}

fn with<T: Clone + Send>(t: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || t.clone())
}

impl Reject for Error {}
