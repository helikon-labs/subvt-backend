use crate::ServiceState;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use futures::FutureExt;
use k256::ecdsa::signature::Verifier;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use subvt_types::app::User;
use subvt_types::err::ServiceError;

enum AuthError {
    PublicKeyMissing,
    InvalidPublicKey,
    SignatureMissing,
    InvalidSignature,
    NonceMissing,
    InvalidNonce,
    InternalError,
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

pub struct AuthService<S> {
    service: Rc<S>,
}

// test with https://paulmillr.com/noble/
impl<S> AuthService<S> {
    async fn authenticate(request: &ServiceRequest) -> Result<(), Error> {
        if !request.path().starts_with("/secure") {
            return Ok(());
        }
        let postgres = if let Some(state) = request.app_data::<Data<ServiceState>>() {
            state.postgres.clone()
        } else {
            return Err(AuthError::InternalError.into());
        };
        let public_key_header = request.headers().get("SubVT-Public-Key");
        let signature_header = request.headers().get("SubVT-Signature");
        let nonce_header = request.headers().get("SubVT-Nonce");
        if public_key_header.is_none() {
            return Err(AuthError::PublicKeyMissing.into());
        }
        if signature_header.is_none() {
            return Err(AuthError::SignatureMissing.into());
        }
        if nonce_header.is_none() {
            return Err(AuthError::NonceMissing.into());
        }
        let public_key_hex = if let Ok(public_key_hex) = public_key_header.unwrap().to_str() {
            public_key_hex
        } else {
            return Err(AuthError::InvalidPublicKey.into());
        };
        let verify_key = if let Some(public_key) =
            hex::decode(public_key_hex.trim_start_matches("0x"))
                .ok()
                .and_then(|bytes| k256::ecdsa::VerifyingKey::from_sec1_bytes(&bytes).ok())
        {
            public_key
        } else {
            return Err(AuthError::InvalidPublicKey.into());
        };
        let signature = if let Some(signature) = signature_header
            .unwrap()
            .to_str()
            .ok()
            .and_then(|hex| hex::decode(&hex.trim_start_matches("0x")).ok())
            .and_then(|bytes| ecdsa::Signature::from_der(&bytes).ok())
        {
            signature
        } else {
            return Err(AuthError::InvalidSignature.into());
        };
        let nonce = if let Some(nonce) = nonce_header
            .unwrap()
            .to_str()
            .ok()
            .and_then(|number_str| number_str.parse::<u64>().ok())
        {
            nonce
        } else {
            return Err(AuthError::InvalidNonce.into());
        };
        let message = format!("{}{}", request.path(), nonce);
        if verify_key.verify(message.as_bytes(), &signature).is_err() {
            return Err(AuthError::InvalidSignature.into());
        }
        // find user and insert into context (if exists)
        if let Ok(maybe_user) = postgres.get_user_by_public_key(public_key_hex).await {
            request
                .extensions_mut()
                .insert::<Rc<Option<User>>>(Rc::new(maybe_user))
        } else {
            return Err(AuthError::InternalError.into());
        };
        Ok(())
    }
}

impl<S, B> Service<ServiceRequest> for AuthService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        async move {
            Self::authenticate(&request).await?;
            let result = service.call(request).await?;
            Ok(result)
        }
        .boxed_local()
    }
}

pub struct AuthServiceFactory;

impl<S, B> Transform<S, ServiceRequest> for AuthServiceFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthService {
            service: Rc::new(service),
        }))
    }
}
