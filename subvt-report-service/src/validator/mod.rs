use crate::{ResultResponse, ServiceState};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::err::ServiceError;
use subvt_types::report::{
    BlockSummary, ValidatorDetailsReport, ValidatorListReport, ValidatorSummaryReport,
};
use subvt_types::subvt::{ValidatorSearchSummary, ValidatorSummary};

fn validate_path_param(ss58_address_or_account_id: &str) -> Result<AccountId, HttpResponse> {
    let account_id = match AccountId::from_str(ss58_address_or_account_id) {
        Ok(account_id) => account_id,
        Err(_) => match AccountId::from_str(ss58_address_or_account_id) {
            Ok(account_id) => account_id,
            Err(_) => {
                return Err(HttpResponse::BadRequest()
                    .json(ServiceError::from("Invalid address or account id.")))
            }
        },
    };
    Ok(account_id)
}

fn get_finalized_block_summary(
    data: &web::Data<ServiceState>,
) -> Result<BlockSummary, HttpResponse> {
    match data.finalized_block_summary.read() {
        Ok(block_summary) => Ok((*block_summary).clone()),
        Err(_) => Err(HttpResponse::InternalServerError().json(ServiceError::from(
            "Internal Error: Cannot get finalized block.",
        ))),
    }
}

fn get_validator_list(
    data: &web::Data<ServiceState>,
    is_active: bool,
) -> Result<Vec<ValidatorSummary>, HttpResponse> {
    let validator_list_lock = if is_active {
        &data.active_validator_list
    } else {
        &data.inactive_validator_list
    };
    match validator_list_lock.read() {
        Ok(list) => Ok((*list).clone()),
        Err(_) => Err(HttpResponse::InternalServerError().json(ServiceError::from(
            "Internal Error: Cannot get validator list.",
        ))),
    }
}

#[derive(Deserialize)]
pub(crate) struct ValidatorPathParameter {
    ss58_address_or_account_id: String,
}

#[get("/validator/{ss58_address_or_account_id}/summary")]
pub(crate) async fn validator_summary_service(
    path: web::Path<ValidatorPathParameter>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let account_id = match validate_path_param(&path.into_inner().ss58_address_or_account_id) {
        Ok(account_id) => account_id,
        Err(response) => return Ok(response),
    };
    let finalized_block = data.redis.get_finalized_block_summary().await?;
    if let Some(validator_details) = data
        .redis
        .fetch_validator_details(finalized_block.number, &account_id)
        .await?
    {
        Ok(HttpResponse::Ok().json(ValidatorSummaryReport {
            finalized_block,
            validator_summary: ValidatorSummary::from(&validator_details),
        }))
    } else {
        Ok(HttpResponse::NotFound().json(ServiceError::from("Validator not found.")))
    }
}

#[get("/validator/{ss58_address_or_account_id}/details")]
pub(crate) async fn validator_details_service(
    path: web::Path<ValidatorPathParameter>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let account_id = match validate_path_param(&path.into_inner().ss58_address_or_account_id) {
        Ok(account_id) => account_id,
        Err(response) => return Ok(response),
    };
    let finalized_block = data.redis.get_finalized_block_summary().await?;
    if let Some(validator_details) = data
        .redis
        .fetch_validator_details(finalized_block.number, &account_id)
        .await?
    {
        Ok(HttpResponse::Ok().json(ValidatorDetailsReport {
            finalized_block,
            validator_details,
        }))
    } else {
        Ok(HttpResponse::NotFound().json(ServiceError::from("Validator not found.")))
    }
}

#[get("/validator/list")]
pub(crate) async fn validator_list_service(data: web::Data<ServiceState>) -> ResultResponse {
    let finalized_block = match get_finalized_block_summary(&data) {
        Ok(block_summary) => block_summary,
        Err(response) => return Ok(response),
    };
    let mut active_validator_list = {
        match get_validator_list(&data, true) {
            Ok(list) => list,
            Err(response) => return Ok(response),
        }
    };
    let mut inactive_validator_list = {
        match get_validator_list(&data, false) {
            Ok(list) => list,
            Err(response) => return Ok(response),
        }
    };
    active_validator_list.append(&mut inactive_validator_list);
    Ok(HttpResponse::Ok().json(ValidatorListReport {
        finalized_block,
        validators: active_validator_list,
    }))
}

#[get("/validator/list/active")]
pub(crate) async fn active_validator_list_service(data: web::Data<ServiceState>) -> ResultResponse {
    let finalized_block = match get_finalized_block_summary(&data) {
        Ok(block_summary) => block_summary,
        Err(response) => return Ok(response),
    };
    let active_validator_list = {
        match get_validator_list(&data, true) {
            Ok(list) => list,
            Err(response) => return Ok(response),
        }
    };
    Ok(HttpResponse::Ok().json(ValidatorListReport {
        finalized_block,
        validators: active_validator_list,
    }))
}

#[get("/validator/list/inactive")]
pub(crate) async fn inactive_validator_list_service(
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let finalized_block = match get_finalized_block_summary(&data) {
        Ok(block_summary) => block_summary,
        Err(response) => return Ok(response),
    };
    let inactive_validator_list = {
        match get_validator_list(&data, false) {
            Ok(list) => list,
            Err(response) => return Ok(response),
        }
    };
    Ok(HttpResponse::Ok().json(ValidatorListReport {
        finalized_block,
        validators: inactive_validator_list,
    }))
}

#[derive(Deserialize)]
pub(crate) struct ValidatorSearchQueryParameters {
    query: String,
}

#[get("/validator/search")]
pub(crate) async fn validator_search_service(
    query: web::Query<ValidatorSearchQueryParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let mut active_validator_list = {
        match get_validator_list(&data, true) {
            Ok(list) => list,
            Err(response) => return Ok(response),
        }
    };
    let mut inactive_validator_list = {
        match get_validator_list(&data, false) {
            Ok(list) => list,
            Err(response) => return Ok(response),
        }
    };
    active_validator_list.append(&mut inactive_validator_list);
    let list: Vec<ValidatorSearchSummary> = active_validator_list
        .iter()
        .filter_map(|validator_summary| {
            if validator_summary.filter(&query.query) {
                Some(ValidatorSearchSummary::from(validator_summary))
            } else {
                None
            }
        })
        .collect();
    Ok(HttpResponse::Ok().json(list))
}
