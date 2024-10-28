use crate::{ResultResponse, ServiceState};
use actix_web::{get, web, HttpResponse};
use subvt_types::err::ServiceError;

pub(crate) mod para;
pub(crate) mod validator;

/// Gets the current era.
#[get("/session/current")]
pub(crate) async fn current_session_service(data: web::Data<ServiceState>) -> ResultResponse {
    if let Some(epoch) = data.postgres.get_current_epoch().await? {
        Ok(HttpResponse::Ok().json(epoch))
    } else {
        Ok(HttpResponse::NotFound().json(ServiceError::from(
            "Current era not found.".to_string().as_ref(),
        )))
    }
}
