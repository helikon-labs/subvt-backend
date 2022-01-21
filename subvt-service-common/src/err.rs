//! Service error types.
use log::error;
use std::fmt::{Display, Formatter};
use subvt_types::err::ServiceError;

#[derive(Debug)]
pub struct InternalServerError {
    err: anyhow::Error,
}

impl Display for InternalServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error!("{:?}", self.err);
        let err = ServiceError::from("Internal server error.");
        write!(f, "{}", serde_json::to_string(&err).unwrap())
    }
}

impl actix_web::error::ResponseError for InternalServerError {}

impl From<anyhow::Error> for InternalServerError {
    fn from(err: anyhow::Error) -> InternalServerError {
        InternalServerError { err }
    }
}
