//! Service common traits and functions.
//! All SubVT services (executables) adhere to this protocol.
use async_trait::async_trait;
use std::str::FromStr;
use subvt_config::Config;
use subvt_types::substrate::Chain;

pub mod err;

#[async_trait(?Send)]
pub trait Service {
    async fn run(&'static self) -> anyhow::Result<()>;

    async fn start(&'static self) {
        let config = Config::default();
        subvt_logging::init(&config);
        log::debug!("Starting service...");
        Chain::from_str(&config.substrate.chain)
            .unwrap()
            .sp_core_set_default_ss58_version();
        let delay_seconds = config.common.recovery_retry_seconds;
        loop {
            let result = self.run().await;
            if let Err(error) = result {
                log::error!("{:?}", error);
            }
            log::error!(
                "Process exited. Will try again in {} seconds.",
                delay_seconds,
            );
            std::thread::sleep(std::time::Duration::from_secs(delay_seconds));
        }
    }
}
