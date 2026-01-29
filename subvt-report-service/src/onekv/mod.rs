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
pub(crate) async fn get_onekv_nominator_summaries(_: web::Data<ServiceState>) -> ResultResponse {
    let nominators: Vec<DNNominatorSummary> = Vec::new();
    Ok(HttpResponse::Ok().json(nominators))
}
