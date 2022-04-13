//!  Public reporting REST services.
use actix_web::dev::Service as _;
use actix_web::web::Data;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use async_trait::async_trait;
use futures_util::future::FutureExt;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::{err::InternalServerError, Service};
use subvt_types::crypto::AccountId;
use subvt_types::err::ServiceError;

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

type ResultResponse = Result<HttpResponse, InternalServerError>;

#[derive(Clone)]
struct ServiceState {
    postgres: Arc<PostgreSQLNetworkStorage>,
}

#[derive(Deserialize)]
struct ValidatorReportPathParameters {
    account_id_hex_string: String,
}

#[derive(Deserialize)]
struct EraReportQueryParameters {
    start_era_index: u32,
    /// Report will be generated for a single era when this parameter is omitted.
    #[serde(rename(deserialize = "end_era_index"))]
    maybe_end_era_index: Option<u32>,
}

/// Gets the report for a certain validator in a range of eras, or a single era.
/// See `EraValidatorReport` struct in the `subvt-types` for details.
#[get("/report/validator/{account_id_hex_string}")]
async fn era_validator_report_service(
    path: web::Path<ValidatorReportPathParameters>,
    query: web::Query<EraReportQueryParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    if let Some(end_era_index) = query.maybe_end_era_index {
        if end_era_index < query.start_era_index {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(
                "End era index cannot be less than start era index.",
            )));
        }
        let era_count = end_era_index - query.start_era_index;
        if era_count > CONFIG.report.max_era_index_range {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(
                format!(
                    "Report cannot span {} eras. Maximum allowed is {}.",
                    era_count, CONFIG.report.max_era_index_range
                )
                .as_ref(),
            )));
        }
    }
    if let Ok(account_id) = AccountId::from_str(&path.account_id_hex_string) {
        Ok(HttpResponse::Ok().json(
            data.postgres
                .get_era_validator_report(
                    query.start_era_index,
                    query.maybe_end_era_index.unwrap_or(query.start_era_index),
                    &account_id.to_string(),
                )
                .await?,
        ))
    } else {
        Ok(HttpResponse::BadRequest().json(ServiceError::from("Invalid account id.")))
    }
}

/// Gets the report for a range of eras, or a single era.
/// See `EraReport` struct in the `subvt-types` definition for details.
#[get("/report/era")]
async fn era_report_service(
    query: web::Query<EraReportQueryParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    if let Some(end_era_index) = query.maybe_end_era_index {
        if end_era_index < query.start_era_index {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(
                "End era index cannot be less than start era index.",
            )));
        }
        let era_count = end_era_index - query.start_era_index;
        if era_count > CONFIG.report.max_era_index_range {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(
                format!(
                    "Report cannot span {} eras. Maximum allowed is {}.",
                    era_count, CONFIG.report.max_era_index_range
                )
                .as_ref(),
            )));
        }
    }
    Ok(HttpResponse::Ok().json(
        data.postgres
            .get_era_report(
                query.start_era_index,
                query.maybe_end_era_index.unwrap_or(query.start_era_index),
            )
            .await?,
    ))
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
                .app_data(Data::new(ServiceState {
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
                .service(era_validator_report_service)
                .service(era_report_service)
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
