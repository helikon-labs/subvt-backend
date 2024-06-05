use crate::{ResultResponse, ServiceState, CONFIG};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use std::str::FromStr;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::err::ServiceError;
use subvt_types::report::{
    BlockSummary, EraValidatorPayoutReport, EraValidatorRewardReport, ValidatorDetailsReport,
    ValidatorListReport, ValidatorSummaryReport, ValidatorTotalRewardChartData,
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

#[get("/validator/{ss58_address_or_account_id}/era/reward")]
pub(crate) async fn validator_era_rewards_service(
    path: web::Path<ValidatorPathParameter>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let account_id = match validate_path_param(&path.into_inner().ss58_address_or_account_id) {
        Ok(account_id) => account_id,
        Err(response) => return Ok(response),
    };
    let era_rewards: Vec<EraValidatorRewardReport> = data
        .postgres
        .get_validator_all_era_rewards(&account_id)
        .await?
        .iter()
        .map(EraValidatorRewardReport::from)
        .collect();
    Ok(HttpResponse::Ok().json(era_rewards))
}

#[get("/validator/{ss58_address_or_account_id}/era/payout")]
pub(crate) async fn validator_era_payouts_service(
    path: web::Path<ValidatorPathParameter>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let account_id = match validate_path_param(&path.into_inner().ss58_address_or_account_id) {
        Ok(account_id) => account_id,
        Err(response) => return Ok(response),
    };
    let era_payouts: Vec<EraValidatorPayoutReport> = data
        .postgres
        .get_validator_all_era_payouts(&account_id)
        .await?
        .iter()
        .map(EraValidatorPayoutReport::from)
        .collect();
    Ok(HttpResponse::Ok().json(era_payouts))
}

#[derive(Deserialize)]
pub(crate) struct ValidatorRewardChartQueryParameters {
    start_timestamp: u64,
    end_timestamp: u64,
}

#[get("/validator/reward/chart")]
pub(crate) async fn validator_reward_chart_service(
    query: web::Query<ValidatorRewardChartQueryParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let people_client = SubstrateClient::new(
        CONFIG.substrate.people_rpc_url.as_str(),
        CONFIG.substrate.network_id,
        CONFIG.substrate.connection_timeout_seconds,
        CONFIG.substrate.request_timeout_seconds,
    )
    .await?;
    let rewards = data
        .postgres
        .get_validator_total_rewards(query.start_timestamp, query.end_timestamp)
        .await?;
    let account_ids: Vec<AccountId> = rewards
        .iter()
        .map(|reward| reward.validator_account_id)
        .collect();
    let mut new_account_ids = vec![];
    {
        let account_cache = data.account_cache.read().unwrap();
        for account_id in &account_ids {
            if !account_cache.contains_key(account_id) {
                new_account_ids.push(*account_id);
            }
        }
    }
    let people_block_hash = people_client.get_current_block_hash().await?;
    let new_accounts = people_client
        .get_accounts(&new_account_ids, false, &people_block_hash)
        .await?;
    let parent_account_ids: Vec<AccountId> = new_accounts
        .iter()
        .filter_map(|account| account.parent_account_id)
        .collect();
    let mut new_parent_account_ids = vec![];
    {
        let account_cache = data.account_cache.read().unwrap();
        for parent_account_id in &parent_account_ids {
            if !account_cache.contains_key(parent_account_id) {
                new_parent_account_ids.push(*parent_account_id);
            }
        }
    }
    let new_parent_accounts = people_client
        .get_accounts(&new_parent_account_ids, false, &people_block_hash)
        .await?;
    // write new accounts to cache
    {
        let mut account_cache = data.account_cache.write().unwrap();
        for new_account in &new_accounts {
            account_cache.insert(new_account.id, new_account.clone());
        }
        for new_parent_account in &new_parent_accounts {
            account_cache.insert(new_parent_account.id, new_parent_account.clone());
        }
    }
    let account_cache = data.account_cache.read().unwrap();
    Ok(HttpResponse::Ok().json(ValidatorTotalRewardChartData {
        accounts: account_cache.values().cloned().collect(),
        rewards,
        start_timestamp: query.start_timestamp,
        end_timestamp: query.end_timestamp,
    }))
}
