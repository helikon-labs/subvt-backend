use actix_web::web::Data;
use actix_web::{test, App};
use std::sync::Arc;
use subvt_app_service::{get_networks, ServiceState};
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::Network;

#[actix_rt::test]
async fn test_get_networks() {
    let config = Config::test().unwrap();
    let postgres = Arc::new(
        PostgreSQLAppStorage::new(&config, config.get_app_postgres_url())
            .await
            .unwrap(),
    );
    let app = test::init_service(
        App::new()
            .app_data(Data::new(ServiceState {
                postgres: postgres.clone(),
            }))
            .service(get_networks),
    )
    .await;
    let request = test::TestRequest::get().uri("/network").to_request();
    let response = test::call_service(&app, request).await;
    let networks: Vec<Network> = test::read_body_json(response).await;
    assert!(!networks.is_empty());
}
