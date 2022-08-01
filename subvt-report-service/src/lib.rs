//!  Public reporting REST services.
#![warn(clippy::disallowed_types)]
use actix_web::dev::Service as _;
use actix_web::{web, App, HttpResponse, HttpServer};
use async_trait::async_trait;
use futures_util::future::FutureExt;
use lazy_static::lazy_static;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::{err::InternalServerError, Service};

mod era;
mod metrics;
mod session;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub(crate) type ResultResponse = Result<HttpResponse, InternalServerError>;

#[derive(Clone)]
pub(crate) struct ServiceState {
    postgres: Arc<PostgreSQLNetworkStorage>,
}

async fn on_server_ready() {
    log::info!("HTTP service started.");
}

#[derive(Default)]
pub struct ReportService;

#[async_trait(?Send)]
impl Service for ReportService {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.report_service_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        let postgres = Arc::new(
            PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?,
        );
        log::info!("Starting HTTP service.");
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(ServiceState {
                    postgres: postgres.clone(),
                }))
                .wrap_fn(|request, service| {
                    metrics::request_counter().inc();
                    metrics::connection_count().inc();
                    let start = std::time::Instant::now();
                    service.call(request).map(move |result| {
                        match &result {
                            Ok(response) => {
                                let status_code = response.response().status();
                                metrics::response_time_ms()
                                    .observe(start.elapsed().as_millis() as f64);
                                metrics::response_status_code_counter(status_code.as_str()).inc();
                            }
                            Err(error) => {
                                let status_code = error.as_response_error().status_code();
                                metrics::response_time_ms()
                                    .observe(start.elapsed().as_millis() as f64);
                                metrics::response_status_code_counter(status_code.as_str()).inc();
                            }
                        }
                        metrics::connection_count().dec();
                        result
                    })
                })
                .service(era::era_validator_report_service)
                .service(era::era_report_service)
                .service(session::validator::session_validator_report_service)
                .service(session::validator::session_validator_para_vote_service)
                .service(session::para::session_paras_vote_summaries_service)
        })
        .workers(10)
        .disable_signals()
        .bind(format!(
            "{}:{}",
            CONFIG.http.service_host, CONFIG.http.report_service_port,
        ))?
        .run();
        let (server_result, _) = tokio::join!(server, on_server_ready());
        Ok(server_result?)
    }
}
