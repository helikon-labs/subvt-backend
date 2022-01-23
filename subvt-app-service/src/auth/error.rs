//! Enum for the authentication errors.
use actix_web::http::StatusCode;
use std::fmt::{Debug, Display, Formatter};
use subvt_types::err::ServiceError;

pub enum AuthError {
    PublicKeyMissing,
    InvalidPublicKey,
    SignatureMissing,
    InvalidSignature,
    NonceMissing,
    InvalidNonce,
    InternalError,
    UserNotFound,
    InvalidBody,
}

impl AuthError {
    fn display(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PublicKeyMissing => {
                write!(
                    f,
                    "{}",
                    serde_json::to_string(&ServiceError::from("Public key header is missing."))
                        .unwrap()
                )
            }
            Self::InvalidPublicKey => {
                write!(
                    f,
                    "{}",
                    serde_json::to_string(&ServiceError::from("Invalid public key header."))
                        .unwrap()
                )
            }
            Self::SignatureMissing => {
                write!(
                    f,
                    "{}",
                    serde_json::to_string(&ServiceError::from("Signature header is missing."))
                        .unwrap()
                )
            }
            Self::InvalidSignature => {
                write!(
                    f,
                    "{}",
                    serde_json::to_string(&ServiceError::from("Invalid signature.")).unwrap()
                )
            }
            Self::NonceMissing => {
                write!(
                    f,
                    "{}",
                    serde_json::to_string(&ServiceError::from("Nonce header is missing.")).unwrap()
                )
            }
            Self::InvalidNonce => {
                write!(
                    f,
                    "{}",
                    serde_json::to_string(&ServiceError::from("Invalid nonce.")).unwrap()
                )
            }
            Self::InternalError => {
                write!(
                    f,
                    "{}",
                    serde_json::to_string(&ServiceError::from("Internal error.")).unwrap()
                )
            }
            Self::UserNotFound => {
                write!(
                    f,
                    "{}",
                    serde_json::to_string(&ServiceError::from("User not found.")).unwrap()
                )
            }
            Self::InvalidBody => {
                write!(
                    f,
                    "{}",
                    serde_json::to_string(&ServiceError::from("Invalid body: UTF-8 error."))
                        .unwrap()
                )
            }
        }
    }
}

impl Debug for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.display(f)
    }
}

impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.display(f)
    }
}

impl actix_web::error::ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::FORBIDDEN,
        }
    }
}
