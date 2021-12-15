use actix_web::web::Data;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::debug;
use serde::Deserialize;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::PostgreSQLStorage;
use subvt_service_common::{err::InternalServerError, Service};
use subvt_types::err::ServiceError;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

type ResultResponse = Result<HttpResponse, InternalServerError>;

#[derive(Clone)]
struct ServiceState {
    postgres: Arc<PostgreSQLStorage>,
}

#[derive(Deserialize)]
struct ValidatorReportPathParameters {
    account_id_hex_string: String,
}

#[derive(Deserialize)]
struct EraReportQueryParameters {
    start_era_index: u32,
    #[serde(rename(deserialize = "end_era_index"))]
    maybe_end_era_index: Option<u32>,
}

#[get("/service/report/validator/{account_id_hex_string}")]
async fn era_validator_report_service(
    path: web::Path<ValidatorReportPathParameters>,
    query: web::Query<EraReportQueryParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    if let Some(end_era_index) = query.maybe_end_era_index {
        if end_era_index < query.start_era_index {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(
                "End era index cannot be less than start era index.".to_string(),
            )));
        }
        let era_count = end_era_index - query.start_era_index;
        if era_count > CONFIG.report.max_era_index_range {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(format!(
                "Report cannot span {} eras. Maximum allowed is {}.",
                era_count, CONFIG.report.max_era_index_range
            ))));
        }
    }
    Ok(HttpResponse::Ok().json(
        data.postgres
            .get_era_validator_report(
                query.start_era_index,
                query.maybe_end_era_index.unwrap_or(query.start_era_index),
                &path.account_id_hex_string,
            )
            .await?,
    ))
}

#[get("/service/report/era")]
async fn era_report_service(
    query: web::Query<EraReportQueryParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    if let Some(end_era_index) = query.maybe_end_era_index {
        if end_era_index < query.start_era_index {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(
                "End era index cannot be less than start era index.".to_string(),
            )));
        }
        let era_count = end_era_index - query.start_era_index;
        if era_count > CONFIG.report.max_era_index_range {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(format!(
                "Report cannot span {} eras. Maximum allowed is {}.",
                era_count, CONFIG.report.max_era_index_range
            ))));
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

#[derive(Default)]
pub struct ReportService;

#[async_trait(?Send)]
impl Service for ReportService {
    async fn run(&'static self) -> anyhow::Result<()> {
        let postgres = Arc::new(PostgreSQLStorage::new(&CONFIG).await?);
        debug!("Starting HTTP service...");
        let result = HttpServer::new(move || {
            App::new()
                .app_data(Data::new(ServiceState {
                    postgres: postgres.clone(),
                }))
                .service(era_validator_report_service)
                .service(era_report_service)
        })
        .workers(10)
        .disable_signals()
        .bind(format!(
            "{}:{}",
            CONFIG.http.host, CONFIG.http.report_service_port,
        ))?
        .run()
        .await;
        Ok(result?)
    }
}
