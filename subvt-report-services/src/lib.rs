use async_trait::async_trait;
use lazy_static::lazy_static;
use subvt_config::Config;
use subvt_service_common::Service;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct ReportServices;

#[async_trait]
impl Service for ReportServices {
    async fn run(&'static self) -> anyhow::Result<()> {
        Ok(())
    }
}
