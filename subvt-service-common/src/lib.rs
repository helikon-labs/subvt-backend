//! Service common traits and functions.
//! All SubVT services (executables) adhere to this protocol.
#![warn(clippy::disallowed_types)]
use async_trait::async_trait;
use std::str::FromStr;
use subvt_config::Config;
use subvt_types::substrate::Chain;

pub mod err;

#[async_trait(?Send)]
pub trait Service {
    fn get_metrics_server_addr() -> (&'static str, u16);

    async fn run(&'static self) -> anyhow::Result<()>;

    async fn start(&'static self) {
        let config = Config::default();
        subvt_logging::init(&config);
        log::info!("Starting service...");
        Chain::from_str(&config.substrate.chain)
            .unwrap()
            .sp_core_set_default_ss58_version();
        tokio::spawn(subvt_metrics::server::start(Self::get_metrics_server_addr()));
        let delay_seconds = config.common.recovery_retry_seconds;
        loop {
            let result = self.run().await;
            if let Err(error) = result {
                log::error!("{error:?}");
            }
            log::error!("Process exited. Will try again in {delay_seconds} seconds.",);
            tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
        }
    }
}
