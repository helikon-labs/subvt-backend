use crate::{ResultResponse, ServiceState};
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use subvt_types::crypto::AccountId;
use subvt_types::substrate::Balance;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DNNominatorSummary {
    pub id: u64,
    pub onekv_id: String,
    pub stash_account_id: AccountId,
    pub stash_address: String,
    pub bonded_amount: Balance,
    pub last_nomination_at: u64,
}

#[get("/onekv/nominator")]
pub(crate) async fn get_onekv_nominator_summaries(data: web::Data<ServiceState>) -> ResultResponse {
    let stash_account_ids = data
        .postgres
        .get_onekv_nominator_stash_account_ids()
        .await?;
    let mut nominators: Vec<DNNominatorSummary> = Vec::new();
    let mut id = 1;
    for stash_account_id in stash_account_ids.iter() {
        nominators.push(DNNominatorSummary {
            id,
            onekv_id: id.to_string(),
            stash_account_id: *stash_account_id,
            stash_address: stash_account_id.to_string(),
            bonded_amount: 0,
            last_nomination_at: 0,
        });
        id += 1;
    }
    Ok(HttpResponse::Ok().json(nominators))
}
