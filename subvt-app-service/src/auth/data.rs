//! Contains the wrapper for the `User` struct.
//! Implements `FromRequest` for the wrapper to be able to pass the authenticated `User`
//! to the request extensions.
use crate::auth::error::AuthError;
use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};
use futures::future::{ready, Ready};
use subvt_types::app::User;

#[derive(Debug)]
pub struct AuthenticatedUser(User);

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let value = request.extensions().get::<User>().cloned();
        let result = match value {
            Some(v) => Ok(AuthenticatedUser(v)),
            None => Err(AuthError::UserNotFound.into()),
        };
        ready(result)
    }
}

impl std::ops::Deref for AuthenticatedUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
