use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::debug;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::PostgreSQLStorage;
use subvt_service_common::Service;
use subvt_types::app::{AppServiceError, User, PUBLIC_KEY_HEX_LENGTH};

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Clone)]
struct ServiceState {
    postgres: Arc<PostgreSQLStorage>,
}

#[get("/service/network")]
async fn get_networks(data: web::Data<ServiceState>) -> impl Responder {
    match data.postgres.get_networks().await {
        Ok(networks) => HttpResponse::Ok().json(networks),
        Err(error) => {
            HttpResponse::InternalServerError().json(AppServiceError::from(format!("{:?}", error)))
        }
    }
}

#[get("/service/notification/channel")]
async fn get_notification_channels(data: web::Data<ServiceState>) -> impl Responder {
    match data.postgres.get_notification_channels().await {
        Ok(channels) => HttpResponse::Ok().json(channels),
        Err(error) => {
            HttpResponse::InternalServerError().json(AppServiceError::from(format!("{:?}", error)))
        }
    }
}

#[get("/service/notification/type")]
async fn get_notification_types(data: web::Data<ServiceState>) -> impl Responder {
    match data.postgres.get_notification_types().await {
        Ok(types) => HttpResponse::Ok().json(types),
        Err(error) => {
            HttpResponse::InternalServerError().json(AppServiceError::from(format!("{:?}", error)))
        }
    }
}

#[post("/service/user")]
async fn create_user(data: web::Data<ServiceState>, mut user: web::Json<User>) -> impl Responder {
    let public_key_hex = user.public_key_hex.trim_start_matches("0x").to_uppercase();
    // validate length
    if public_key_hex.len() != PUBLIC_KEY_HEX_LENGTH {
        return HttpResponse::BadRequest().json(AppServiceError::from(format!(
            "Public key should be {} characters long hexadecimal string.",
            PUBLIC_KEY_HEX_LENGTH
        )));
    }
    // validate hex
    if hex::decode(&public_key_hex).is_err() {
        return HttpResponse::BadRequest().json(AppServiceError::from(
            "Public key should be valid hexadecimal string.".to_string(),
        ));
    }
    // check duplicate
    match data.postgres.user_exists(&public_key_hex).await {
        Ok(user_exists) => {
            if user_exists {
                return HttpResponse::Conflict().json(AppServiceError::from(
                    "A user exists with the given public key.".to_string(),
                ));
            }
        }
        Err(error) => {
            return HttpResponse::InternalServerError()
                .json(AppServiceError::from(format!("{:?}", error)));
        }
    }
    user.public_key_hex = format!("0x{}", public_key_hex);
    let save_result = &data.postgres.save_user(&user).await;
    match save_result {
        Ok(id) => {
            user.id = *id;
            HttpResponse::Ok().json(user)
        }
        Err(error) => {
            HttpResponse::InternalServerError().json(AppServiceError::from(format!("{:?}", error)))
        }
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
