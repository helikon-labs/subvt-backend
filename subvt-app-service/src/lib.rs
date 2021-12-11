use actix_web::web::Data;
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::debug;
use serde::Deserialize;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::PostgreSQLStorage;
use subvt_service_common::{err::InternalServerError, Service};
use subvt_types::app::{AppServiceError, User, UserNotificationChannel, PUBLIC_KEY_HEX_LENGTH};

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

type ResultResponse = Result<HttpResponse, InternalServerError>;

#[derive(Clone)]
struct ServiceState {
    postgres: Arc<PostgreSQLStorage>,
}

#[get("/service/network")]
async fn get_networks(state: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_networks().await?))
}

#[get("/service/notification/channel")]
async fn get_notification_channels(state: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_notification_channels().await?))
}

#[get("/service/notification/type")]
async fn get_notification_types(state: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_notification_types().await?))
}

#[post("/service/user")]
async fn create_user(state: web::Data<ServiceState>, mut user: web::Json<User>) -> ResultResponse {
    let public_key_hex = user.public_key_hex.trim_start_matches("0x").to_uppercase();
    // validate length
    if public_key_hex.len() != PUBLIC_KEY_HEX_LENGTH {
        return Ok(
            HttpResponse::BadRequest().json(AppServiceError::from(format!(
                "Public key should be {} characters long hexadecimal string.",
                PUBLIC_KEY_HEX_LENGTH
            ))),
        );
    }
    // validate hex
    if hex::decode(&public_key_hex).is_err() {
        return Ok(HttpResponse::BadRequest().json(AppServiceError::from(
            "Public key should be valid hexadecimal string.".to_string(),
        )));
    }
    user.public_key_hex = format!("0x{}", public_key_hex);
    // check duplicate public key
    if state
        .postgres
        .user_exists_with_public_key(&user.public_key_hex)
        .await?
    {
        return Ok(HttpResponse::Conflict().json(AppServiceError::from(
            "A user exists with the given public key.".to_string(),
        )));
    }
    user.id = state.postgres.save_user(&user).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[derive(Deserialize)]
struct UserIdPathParameter {
    pub user_id: u32,
}

#[get("/service/user/{user_id}/notification/channel")]
async fn get_user_notification_channels(
    path_params: web::Path<UserIdPathParameter>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    Ok(HttpResponse::Ok().json(
        state
            .postgres
            .get_user_notification_channels(path_params.user_id)
            .await?,
    ))
}

#[post("/service/user/{user_id}/notification/channel")]
async fn add_user_notification_channel(
    path_params: web::Path<UserIdPathParameter>,
    mut input: web::Json<UserNotificationChannel>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    input.user_id = path_params.user_id as u32;
    // check user exists
    if !state.postgres.user_exists_with_id(input.user_id).await? {
        return Ok(
            HttpResponse::NotFound().json(AppServiceError::from("User not found.".to_string()))
        );
    }
    if !state
        .postgres
        .notification_channel_exists(&input.channel_name)
        .await?
    {
        return Ok(HttpResponse::NotFound().json(AppServiceError::from(
            "Notification channel not found.".to_string(),
        )));
    }
    if state
        .postgres
        .user_notification_channel_target_exists(&input)
        .await?
    {
        return Ok(HttpResponse::Conflict().json(AppServiceError::from(
            "This target exists for the user.".to_string(),
        )));
    }
    // validate input
    if input.target.is_empty() {
        return Ok(HttpResponse::BadRequest().json(AppServiceError::from(
            "Invalid notification target.".to_string(),
        )));
    }
    input.id = state
        .postgres
        .save_user_notification_channel(&input)
        .await?;
    Ok(HttpResponse::Ok().json(input))
}

#[derive(Deserialize)]
struct UserNotificationChannelIdPathParameter {
    pub user_id: u32,
    pub channel_id: u32,
}

#[delete("/service/user/{user_id}/notification/channel/{channel_id}")]
async fn delete_user_notification_channel(
    path_params: web::Path<UserNotificationChannelIdPathParameter>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    let channel_exists = state
        .postgres
        .user_notification_channel_exists(path_params.user_id, path_params.channel_id)
        .await?;
    if !channel_exists {
        return Ok(HttpResponse::NotFound().json(AppServiceError::from(
            "User notification channel not found.".to_string(),
        )));
    }
    match state
        .postgres
        .delete_user_notification_channel(path_params.channel_id)
        .await?
    {
        true => Ok(HttpResponse::NoContent().finish()),
        false => Ok(
            HttpResponse::InternalServerError().json(AppServiceError::from(
                "There was an error deleting the notification channel.".to_string(),
            )),
        ),
    }
}

#[derive(Default)]
pub struct AppService;

#[async_trait(?Send)]
impl Service for AppService {
    async fn run(&'static self) -> anyhow::Result<()> {
        let postgres = Arc::new(PostgreSQLStorage::new(&CONFIG).await?);
        debug!("Starting HTTP service...");
        let result = HttpServer::new(move || {
            App::new()
                .app_data(Data::new(ServiceState {
                    postgres: postgres.clone(),
                }))
                .service(get_networks)
                .service(get_notification_channels)
                .service(get_notification_types)
                .service(create_user)
                .service(add_user_notification_channel)
                .service(get_user_notification_channels)
                .service(delete_user_notification_channel)
        })
        .workers(10)
        .disable_signals()
        .bind(format!(
            "{}:{}",
            CONFIG.http.host, CONFIG.http.app_service_port,
        ))?
        .run()
        .await;
        Ok(result?)
    }
}
