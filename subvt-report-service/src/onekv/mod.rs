use crate::{ResultResponse, ServiceState};
use actix_web::{get, web, HttpResponse};

#[get("/report/onekv/nominator")]
pub(crate) async fn get_onekv_nominator_summaries(data: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(data.postgres.get_onekv_nominator_summaries().await?))
}
