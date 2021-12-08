use actix_web::web::Data;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::debug;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::PostgreSQLStorage;
use subvt_service_common::Service;
use subvt_types::app::AppServiceError;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Clone)]
struct ServiceState {
    postgres: Arc<PostgreSQLStorage>,
}

#[get("/service/network")]
async fn get_networks(data: web::Data<ServiceState>) -> impl Responder {
    let query_result = &data.postgres.get_networks().await;
    match query_result {
        Ok(networks) => HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(networks).unwrap()),
        Err(error) => HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(
                serde_json::to_string(&AppServiceError {
                    description: format!("{:?}", error),
                })
                .unwrap(),
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
