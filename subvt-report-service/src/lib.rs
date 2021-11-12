use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use async_trait::async_trait;
use lazy_static::lazy_static;
use subvt_config::Config;
use subvt_service_common::Service;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Default)]
pub struct ReportService;

#[async_trait(?Send)]
impl Service for ReportService {
    async fn run(&'static self) -> anyhow::Result<()> {
        let result = HttpServer::new(|| App::new().service(hello))
            .bind(format!(
                "{}:{}",
                CONFIG.http.host, CONFIG.http.report_service_port,
            ))?
            .run()
            .await;
        Ok(result?)
    }
}
