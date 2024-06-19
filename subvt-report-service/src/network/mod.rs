use crate::{ResultResponse, ServiceState};
use actix_web::{get, web, HttpResponse};

#[get("/network/status")]
pub(crate) async fn get_network_status(data: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(data.redis.get_network_status().await?))
}