use async_trait::async_trait;
use frankenstein::api_traits::AsyncTelegramApi;
use frankenstein::api_traits::ErrorResponse;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs::File;

static BASE_API_URL: &str = "https://api.telegram.org/bot";

#[derive(PartialEq, Debug, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
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

#[derive(PartialEq, Debug, Serialize, Deserialize, thiserror::Error)]
#[error("Http Error {code}: {message}")]
pub struct HttpError {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct AsyncApi {
    pub api_url: String,
    client: reqwest::Client,
}

impl AsyncApi {
    pub fn new(api_key: &str) -> Self {
        let api_url = format!("{}{}", BASE_API_URL, api_key);
        let client = reqwest::Client::new();
        Self { api_url, client }
    }

    pub fn encode_params<T: serde::ser::Serialize + std::fmt::Debug>(
        params: &T,
    ) -> Result<String, Error> {
        serde_json::to_string(params).map_err(|e| Error::Encode(format!("{:?} : {:?}", e, params)))
    }

    pub async fn decode_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, Error> {
        let status_code = response.status().as_u16();
        match response.text().await {
            Ok(message) => {
                if status_code == 200 {
                    let success_response: T = Self::parse_json(&message)?;
                    return Ok(success_response);
                }

                let error_response: ErrorResponse = Self::parse_json(&message)?;
                Err(Error::Api(error_response))
            }
            Err(e) => {
                let err = Error::Decode(format!("Failed to decode response: {:?}", e));
                Err(err)
            }
        }
    }

    fn parse_json<T: serde::de::DeserializeOwned>(body: &str) -> Result<T, Error> {
        let json_result: Result<T, serde_json::Error> = serde_json::from_str(body);

        match json_result {
            Ok(result) => Ok(result),

            Err(e) => {
                let err = Error::Decode(format!("{:?} : {:?}", e, body));
                Err(err)
            }
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        let message = error.to_string();
        let code = if let Some(status_code) = error.status() {
            status_code.as_u16()
        } else {
            500
        };

        let error = HttpError { code, message };
        Self::Http(error)
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
        use hyper::body::Buf;
        use hyper::Client as HyperClient;

        let url = format!("{}/{}", self.api_url, method);
        println!("URL {}", url);

        let mut prepared_request = self
            .client
            .post(url.clone())
            .header("Content-Type", "application/json");

        prepared_request = if let Some(data) = params {
            println!("ENTER");
            let json_string = Self::encode_params(&data)?;

            let https = hyper_tls::HttpsConnector::new();
            let hyper_client = HyperClient::builder().build::<_, hyper::Body>(https);
            let req = hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri(&url)
                .header("content-type", "application/json")
                .body(hyper::Body::from(json_string))
                .unwrap();
            let resp = hyper_client.request(req).await.unwrap();
            let body = hyper::body::aggregate(resp).await.unwrap();
            let deser_resp: T2 = serde_json::from_reader(body.reader()).unwrap();
            return Ok(deser_resp);

            // prepared_request.body(json_string)
        } else {
            prepared_request
        };

        let response = prepared_request.send().await?;
        let parsed_response: T2 = Self::decode_response(response).await?;

        Ok(parsed_response)
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
        // https://stackoverflow.com/questions/51397872/how-to-post-an-image-using-multipart-form-data-with-hyper
        let json_string = Self::encode_params(&params)?;
        let json_struct: Value = serde_json::from_str(&json_string).unwrap();
        let file_keys: Vec<&str> = files.iter().map(|(key, _)| *key).collect();
        let files_with_paths: Vec<(String, &str, String)> = files
            .iter()
            .map(|(key, path)| {
                (
                    (*key).to_string(),
                    path.to_str().unwrap(),
                    path.file_name().unwrap().to_str().unwrap().to_string(),
                )
            })
            .collect();

        let mut form = multipart::Form::new();
        for (key, val) in json_struct.as_object().unwrap().iter() {
            if !file_keys.contains(&key.as_str()) {
                let val = match val {
                    &Value::String(ref val) => val.to_string(),
                    other => other.to_string(),
                };

                form = form.text(key.clone(), val.clone());
            }
        }

        for (parameter_name, file_path, file_name) in files_with_paths {
            let file = File::open(file_path).await?;

            let part = multipart::Part::stream(file).file_name(file_name);
            form = form.part(parameter_name, part);
        }

        let url = format!("{}/{}", self.api_url, method);

        let response = self.client.post(url).multipart(form).send().await?;
        let parsed_response: T2 = Self::decode_response(response).await?;

        Ok(parsed_response)
    }
}
