use std::fmt::{Display, Formatter};
use subvt_types::app::AppServiceError;

#[derive(Debug)]
pub struct InternalServerError {
    err: anyhow::Error,
}

impl Display for InternalServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err = AppServiceError::from(format!("{:?}", self.err));
        write!(f, "{}", serde_json::to_string(&err).unwrap())
    }
}

impl actix_web::error::ResponseError for InternalServerError {}

impl From<anyhow::Error> for InternalServerError {
    fn from(err: anyhow::Error) -> InternalServerError {
        InternalServerError { err }
    }
}
