use crate::util::{validate_account_id_param, validate_block_hash};
use crate::{ResultResponse, ServiceState};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use subvt_types::err::ServiceError;
use subvt_types::report::{Bond, Controller};

#[derive(Deserialize)]
pub(crate) struct AccountIdPathParameter {
    ss58_address_or_account_id: String,
}

#[derive(Deserialize)]
pub(crate) struct BlockHashQueryParameter {
    block_hash: Option<String>,
}

#[get("/staking/{ss58_address_or_account_id}/controller")]
pub(crate) async fn controller_service(
    path: web::Path<AccountIdPathParameter>,
    query: web::Query<BlockHashQueryParameter>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let stash_account_id =
        match validate_account_id_param(&path.into_inner().ss58_address_or_account_id) {
            Ok(account_id) => account_id,
            Err(response) => return Ok(response),
        };

    if let Some(block_hash) = &query.block_hash {
        if let Err(response) = validate_block_hash(block_hash) {
            return Ok(response);
        }
    }
    let controller_account_id =
        match data
            .substrate_client
            .get_controller_account_id(&stash_account_id, query.block_hash.as_deref())
            .await?
        {
            Some(controller_account_id) => controller_account_id,
            None => return Ok(HttpResponse::NotFound().json(ServiceError::from(
                "No controller account found for the given stash account at the given block hash.",
            ))),
        };
    Ok(HttpResponse::Ok().json(Controller {
        controller_account_id,
        controller_address: controller_account_id.to_ss58_check(),
    }))
}

#[get("/staking/{ss58_address_or_account_id}/bond")]
pub(crate) async fn bond_service(
    path: web::Path<AccountIdPathParameter>,
    query: web::Query<BlockHashQueryParameter>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let controller_account_id =
        match validate_account_id_param(&path.into_inner().ss58_address_or_account_id) {
            Ok(account_id) => account_id,
            Err(response) => return Ok(response),
        };
    if let Some(block_hash) = &query.block_hash {
        if let Err(response) = validate_block_hash(block_hash) {
            return Ok(response);
        }
    }
    let bond = match data.substrate_client.get_stake(
        &controller_account_id,
        query.block_hash.as_deref(),
    ).await? {
        Some(bond) => bond,
        None => return Ok(HttpResponse::NotFound()
            .json(ServiceError::from("No bond found for the controller account of the given stash account at the given block hash.")))
    };
    Ok(HttpResponse::Ok().json(Bond {
        controller_account_id,
        controller_address: controller_account_id.to_ss58_check(),
        bond,
    }))
}
