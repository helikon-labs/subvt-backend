//! Authentication service and factory (`Transform`).
use crate::auth::error::AuthError;
use crate::ServiceState;
use actix_http::h1::Payload;
use actix_web::web::{BytesMut, Data};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use futures::{FutureExt, StreamExt};
use libsecp256k1::{verify, Message, PublicKey, PublicKeyFormat, Signature};
use sha2::{Digest, Sha256};
use std::rc::Rc;
use subvt_types::app::User;

pub struct AuthService<S> {
    service: Rc<S>,
}

impl<S> AuthService<S> {
    async fn authenticate(request: &mut ServiceRequest) -> Result<(), Error> {
        if !request.path().starts_with("/secure") {
            return Ok(());
        }
        let postgres = if let Some(state) = request.app_data::<Data<ServiceState>>() {
            state.postgres.clone()
        } else {
            return Err(AuthError::InternalError.into());
        };
        let public_key_header = if let Some(header) = request.headers().get("SubVT-Public-Key") {
            header
        } else {
            return Err(AuthError::PublicKeyMissing.into());
        };
        let signature_header = if let Some(header) = request.headers().get("SubVT-Signature") {
            header
        } else {
            return Err(AuthError::SignatureMissing.into());
        };
        let nonce_header = if let Some(header) = request.headers().get("SubVT-Nonce") {
            header
        } else {
            return Err(AuthError::NonceMissing.into());
        };
        // extract public key
        let public_key_hex = if let Ok(public_key_hex) = public_key_header.to_str() {
            format!(
                "0x{}",
                public_key_hex.trim_start_matches("0x").to_uppercase()
            )
        } else {
            return Err(AuthError::InvalidPublicKey.into());
        };
        let public_key = if let Some(public_key) =
            hex::decode(public_key_hex.trim_start_matches("0x"))
                .ok()
                .and_then(|bytes| {
                    PublicKey::parse_slice(&bytes, Some(PublicKeyFormat::Compressed)).ok()
                }) {
            public_key
        } else {
            return Err(AuthError::InvalidPublicKey.into());
        };
        // extract signature
        let signature = if let Some(signature) = signature_header
            .to_str()
            .ok()
            .and_then(|hex| hex::decode(&hex.trim_start_matches("0x")).ok())
            .and_then(|bytes| Signature::parse_der(&bytes).ok())
        {
            signature
        } else {
            return Err(AuthError::InvalidSignature.into());
        };
        // extract nonce
        let nonce = if let Some(nonce) = nonce_header
            .to_str()
            .ok()
            .and_then(|number_str| number_str.parse::<u64>().ok())
        {
            nonce
        } else {
            return Err(AuthError::InvalidNonce.into());
        };
        // extract body
        let mut request_body = BytesMut::new();
        while let Some(chunk) = request.take_payload().next().await {
            request_body.extend_from_slice(&chunk?);
        }
        let body = if let Ok(body) = String::from_utf8(request_body.to_vec()) {
            body
        } else {
            return Err(AuthError::InvalidBody.into());
        };
        let mut original_payload = Payload::create(true).1;
        original_payload.unread_data(request_body.freeze());
        request.set_payload(actix_http::Payload::from(original_payload));
        // verify signature
        let message_to_sign = format!(
            "{}{}{}{}",
            request.method().as_str(),
            request.path(),
            body,
            nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(message_to_sign.as_bytes());
        let hash = hasher.finalize();
        let message = Message::parse_slice(&hash).unwrap();
        if !verify(&message, &signature, &public_key) {
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

    fn call(&self, mut request: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        async move {
            Self::authenticate(&mut request).await?;
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
