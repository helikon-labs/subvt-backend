use crate::{ResultResponse, ServiceState, CONFIG};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::err::ServiceError;

#[derive(Deserialize)]
pub(crate) struct ValidatorReportPathParameters {
    account_id_hex_string: String,
}

#[derive(Deserialize)]
pub(crate) struct EraReportQueryParameters {
    start_era_index: u32,
    /// Report will be generated for a single era when this parameter is omitted.
    #[serde(rename(deserialize = "end_era_index"))]
    maybe_end_era_index: Option<u32>,
}

/// Gets the report for a certain validator in a range of eras, or a single era.
/// See `EraValidatorReport` struct in the `subvt-types` for details.
#[get("/report/validator/{account_id_hex_string}")]
pub(crate) async fn era_validator_report_service(
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
pub(crate) async fn era_report_service(
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
