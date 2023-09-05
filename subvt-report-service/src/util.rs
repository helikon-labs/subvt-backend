use actix_web::HttpResponse;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::err::ServiceError;

pub(crate) fn validate_account_id_param(
    ss58_address_or_account_id: &str,
) -> Result<AccountId, HttpResponse> {
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

pub(crate) fn validate_block_hash(block_hash: &str) -> Result<(), HttpResponse> {
    let trimmed_hex_string = block_hash.trim_start_matches("0x");
    if trimmed_hex_string.len() != 64 {
        return Err(HttpResponse::BadRequest().json(ServiceError::from("Invalid block hash.")));
    }
    if hex::decode(trimmed_hex_string).is_err() {
        return Err(HttpResponse::BadRequest().json(ServiceError::from("Invalid block hash.")));
    }
    Ok(())
}
