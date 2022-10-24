use crate::{ResultResponse, ServiceState, CONFIG};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::err::ServiceError;
use subvt_types::report::{EraValidatorListReport, EraValidatorReport};

#[derive(Deserialize)]
pub(crate) struct EraValidatorListReportPathParameters {
    era_index: u32,
}

async fn get_era_validator_list_report(
    data: web::Data<ServiceState>,
    era_index: u32,
    is_active: bool,
) -> ResultResponse {
    let era = match data.postgres.get_era(era_index).await? {
        Some(era) => era,
        None => {
            return Ok(HttpResponse::NotFound().json(ServiceError::from(&format!(
                "Era {} not found.",
                era_index
            ))));
        }
    };
    let account_ids = data
        .postgres
        .get_era_validator_account_ids(era_index, is_active)
        .await?;
    let mut era_validator_reports = vec![];
    for account_id in account_ids {
        if let Some(era_validator_report) = data
            .postgres
            .get_single_era_validator_report(era_index, &account_id.to_string())
            .await?
        {
            era_validator_reports.push(EraValidatorReport{
                account_id: Some(account_id),
                address: Some(account_id.to_ss58_check()),
                era: None,
                ..era_validator_report
            });
        }
    }
    Ok(HttpResponse::Ok().json(EraValidatorListReport {
        era,
        validators: era_validator_reports,
    }))
}

#[get("/report/era/{era_index}/validator/active")]
pub(crate) async fn era_active_validator_list_report_service(
    path: web::Path<EraValidatorListReportPathParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    get_era_validator_list_report(data, path.era_index, true).await
}

#[get("/report/era/{era_index}/validator/inactive")]
pub(crate) async fn era_inactive_validator_list_report_service(
    path: web::Path<EraValidatorListReportPathParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    get_era_validator_list_report(data, path.era_index, false).await
}

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
