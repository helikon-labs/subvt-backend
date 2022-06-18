//! Frankenstein Telegram async API trait implementation. Mostly adopted from the original
//! implementation in the original crate @ https://github.com/ayrat555/frankenstein/blob/master/src/api/async_telegram_api_impl.rs.
use async_trait::async_trait;
use frankenstein::api_traits::AsyncTelegramApi;
use frankenstein::api_traits::ErrorResponse;
use hyper::{body::Buf, client::HttpConnector, Body, Client, Request, Response};
use hyper_multipart_rfc7578::client::multipart::{Body as MultipartBody, Form as MultipartForm};
use hyper_tls::HttpsConnector;
use serde_json::Value;
use std::path::PathBuf;

static BASE_API_URL: &str = "https://api.telegram.org/bot";

#[derive(Eq, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Http(HttpError),
    #[error("Api Error {0:?}")]
    Api(ErrorResponse),
    #[error("Decode Error {0}")]
    Decode(String),
    #[error("Encode Error {0}")]
    Encode(String),
}

#[derive(Eq, PartialEq, Debug, thiserror::Error)]
#[error("Http Error {code}: {message}")]
pub struct HttpError {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct AsyncApi {
    api_url: String,
    client: Client<HttpsConnector<HttpConnector>, Body>,
    multipart_client: Client<HttpsConnector<HttpConnector>, MultipartBody>,
}

impl AsyncApi {
    pub fn new(api_key: &str) -> Self {
        let api_url = format!("{}{}", BASE_API_URL, api_key);
        Self {
            api_url,
            client: Client::builder().build(HttpsConnector::new()),
            multipart_client: Client::builder().build(HttpsConnector::new()),
        }
    }

    pub fn encode_params<T: serde::ser::Serialize + std::fmt::Debug>(
        params: &T,
    ) -> Result<String, Error> {
        serde_json::to_string(params).map_err(|e| Error::Encode(format!("{:?} : {:?}", e, params)))
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        response: Response<Body>,
    ) -> Result<T, Error> {
        let is_successful = response.status().is_success();
        let response_body = hyper::body::aggregate(response).await?;
        if is_successful {
            Ok(serde_json::from_reader(response_body.reader())
                .map_err(|e| Error::Decode(format!("{:?}", e)))?)
        } else {
            Err(Error::Api(
                serde_json::from_reader(response_body.reader())
                    .map_err(|e| Error::Decode(format!("{:?}", e)))?,
            ))
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Self::Http(HttpError {
            code: 500,
            message: error.to_string(),
        })
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        let message = error.to_string();
        Self::Encode(message)
    }
}

#[async_trait]
impl AsyncTelegramApi for AsyncApi {
    type Error = Error;

    async fn request<
        T1: serde::ser::Serialize + std::fmt::Debug + std::marker::Send,
        T2: serde::de::DeserializeOwned,
    >(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<T2, Self::Error> {
        let url = format!("{}/{}", self.api_url, method);
        let request = Request::builder()
            .method(hyper::Method::POST)
            .uri(&url)
            .header("Content-Type", "application/json")
            .body(if let Some(data) = params {
                let body_json = Self::encode_params(&data)?;
                Body::from(body_json)
            } else {
                Body::empty()
            })
            .map_err(|e| Error::Encode(e.to_string()))?;
        let response = self.client.request(request).await?;
        Self::parse_response(response).await
    }

    async fn request_with_form_data<
        T1: serde::ser::Serialize + std::fmt::Debug + std::marker::Send,
        T2: serde::de::DeserializeOwned,
    >(
        &self,
        method: &str,
        params: T1,
        files: Vec<(&str, PathBuf)>,
    ) -> Result<T2, Self::Error> {
        let url = format!("{}/{}", self.api_url, method);
        let mut form = MultipartForm::default();
        let json_string = Self::encode_params(&params)?;
        let json_struct: Value =
            serde_json::from_str(&json_string).map_err(|e| Error::Encode(format!("{:?}", e)))?;
        let mut file_keys = vec![];
        for (key, path) in &files {
            file_keys.push(*key);
            form.add_file(key.to_string(), path.to_str().unwrap())?;
        }
        for (key, val) in json_struct.as_object().unwrap().iter() {
            if !file_keys.contains(&key.as_str()) {
                let value = match val {
                    &Value::String(ref val) => val.to_string(),
                    other => other.to_string(),
                };
                form.add_text(key, value);
            }
        }
        let req_builder = Request::post(url);
        let request = form
            .set_body::<MultipartBody>(req_builder)
            .map_err(|e| Error::Encode(format!("{:?}", e)))?;
        let response = self.multipart_client.request(request).await?;
        Self::parse_response(response).await
    }
}
