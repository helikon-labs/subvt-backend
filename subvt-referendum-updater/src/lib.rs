#![warn(clippy::disallowed_types)]
use async_trait::async_trait;
use enum_iterator::all;
use lazy_static::lazy_static;
use subvt_config::Config;
use subvt_governance::polkassembly::fetch_track_referenda;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;
use subvt_types::governance::track::Track;

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct ReferendumUpdater {}

#[async_trait(?Send)]
impl Service for ReferendumUpdater {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.onekv_updater_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!(
            "Referendum updater has started with {} seconds refresh wait period.",
            CONFIG.referendum_updater.refresh_seconds
        );
        let postgres =
            PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
        let tracks = all::<Track>().collect::<Vec<_>>();

        let _ = postgres.get_open_referenda(None).await?;
        loop {
            metrics::last_run_timestamp_ms().set(chrono::Utc::now().timestamp_millis());
            for track in &tracks {
                log::info!("Fetch {} referenda.", track.name());
                let mut page = 1;
                let limit = 100;
                loop {
                    match fetch_track_referenda(track.id(), page, limit).await {
                        Ok(referenda) => {
                            log::info!("{} referenda in {}.", referenda.len(), track.name());
                            for referendum in &referenda {
                                postgres.save_or_update_referendum(referendum).await?;
                                log::info!("Persisted #{}", referendum.post_id);
                            }
                            if referenda.is_empty() {
                                break;
                            }
                            page += 1;
                        }
                        Err(error) => {
                            log::error!("Error while fetching referenda: {:?}", error);
                            break;
                        }
                    }
                }
            }
            log::info!(
                "Sleep for {} seconds.",
                CONFIG.referendum_updater.refresh_seconds
            );
            tokio::time::sleep(std::time::Duration::from_secs(
                CONFIG.referendum_updater.refresh_seconds,
            ))
            .await;
        }
    }
}
