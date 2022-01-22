//! Authentication service and factory (`Transform`).
use crate::auth::error::AuthError;
use crate::ServiceState;
use actix_web::web::Data;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use futures::FutureExt;
use k256::ecdsa::signature::Verifier;
use std::rc::Rc;
use subvt_types::app::User;

pub struct AuthService<S> {
    service: Rc<S>,
}

/// Test with https://paulmillr.com/noble/.
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
            format!(
                "0x{}",
                public_key_hex.trim_start_matches("0x").to_uppercase()
            )
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
        if let Ok(maybe_user) = postgres.get_user_by_public_key(&public_key_hex).await {
            if let Some(user) = maybe_user {
                request.extensions_mut().insert::<User>(user);
            } else if request.path() != "/secure/user" {
                return Err(AuthError::UserNotFound.into());
            }
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
