//! Error types.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ServiceError {
    pub description: String,
}

impl ServiceError {
    pub fn from(description: String) -> ServiceError {
        ServiceError { description }
    }
}
