use crate::{ResultResponse, ServiceState};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::err::ServiceError;
use subvt_types::report::{ValidatorDetailsReport, ValidatorListReport, ValidatorSummaryReport};
use subvt_types::subvt::ValidatorSummary;

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
    let finalized_block = data.redis.get_finalized_block_summary().await?;
    let mut validators = data
        .redis
        .get_validator_list(finalized_block.number, true)
        .await?;
    let mut inactive_validators = data
        .redis
        .get_validator_list(finalized_block.number, false)
        .await?;
    validators.append(&mut inactive_validators);
    Ok(HttpResponse::Ok().json(ValidatorListReport {
        finalized_block,
        validators,
    }))
}

#[get("/validator/list/active")]
pub(crate) async fn active_validator_list_service(data: web::Data<ServiceState>) -> ResultResponse {
    let finalized_block = data.redis.get_finalized_block_summary().await?;
    Ok(HttpResponse::Ok().json(ValidatorListReport {
        finalized_block: finalized_block.clone(),
        validators: data
            .redis
            .get_validator_list(finalized_block.number, true)
            .await?,
    }))
}

#[get("/validator/list/inactive")]
pub(crate) async fn inactive_validator_list_service(
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let finalized_block = data.redis.get_finalized_block_summary().await?;
    Ok(HttpResponse::Ok().json(ValidatorListReport {
        finalized_block: finalized_block.clone(),
        validators: data
            .redis
            .get_validator_list(finalized_block.number, false)
            .await?,
    }))
}
