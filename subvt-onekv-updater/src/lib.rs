//! Updates the complete 1KV data for the network (only Polkadot and Kusama) on the database.
#![warn(clippy::disallowed_types)]
use async_trait::async_trait;
use lazy_static::lazy_static;
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;
use subvt_types::dn::DNDataResponse;

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub struct OneKVUpdater {
    http_client: reqwest::Client,
}

impl Default for OneKVUpdater {
    fn default() -> Self {
        let http_client: reqwest::Client = reqwest::Client::builder()
            .gzip(true)
            .brotli(true)
            .timeout(std::time::Duration::from_secs(
                CONFIG.http.request_timeout_seconds,
            ))
            .build()
            .unwrap();
        Self { http_client }
    }
}

impl OneKVUpdater {
    async fn update_data(&self, postgres: &PostgreSQLNetworkStorage) -> anyhow::Result<()> {
        log::info!("Fetch DN data.");
        metrics::last_candidate_list_fetch_timestamp_ms()
            .set(chrono::Utc::now().timestamp_millis());
        let start = std::time::Instant::now();
        let response = self
            .http_client
            .get(&CONFIG.dn.data_endpoint)
            .send()
            .await?;
        metrics::nominator_list_fetch_time_ms().observe(start.elapsed().as_millis() as f64);
        metrics::candidate_list_fetch_time_ms().observe(start.elapsed().as_millis() as f64);
        let data: DNDataResponse = response.json().await?;
        // set metrics
        metrics::last_candidate_list_fetch_success_status().set(1);
        metrics::last_candidate_list_successful_fetch_timestamp_ms()
            .set(chrono::Utc::now().timestamp_millis());
        metrics::last_candidate_count().set(data.selected.len() as i64);
        metrics::last_nominator_list_fetch_success_status().set(1);
        metrics::last_nominator_list_successful_fetch_timestamp_ms()
            .set(chrono::Utc::now().timestamp_millis());
        metrics::last_nominator_count().set(data.nominators.len() as i64);
        log::info!("Fetched {} candidates. Save them.", data.selected.len());
        // save candidates
        let mut success_count = 0;
        let mut error_count = 0;
        for (index, node) in data.selected.iter().enumerate() {
            let save_result = postgres
                .save_dn_node(node, CONFIG.dn.candidate_history_record_count as i64)
                .await;
            match save_result {
                Ok(_) => {
                    success_count += 1;
                    log::info!(
                        "Persisted node {} of {} :: {}.",
                        index + 1,
                        data.selected.len(),
                        node.stash,
                    );
                }
                Err(error) => {
                    error_count += 1;
                    log::error!(
                        "Error while persisting details of candidate {}:{:?}",
                        node.stash,
                        error
                    );
                }
            }
        }
        // delete stale records
        postgres
            .delete_onekv_candidate_records_older_than_days(1)
            .await?;
        metrics::last_candidate_persist_success_count().set(success_count);
        metrics::last_candidate_persist_error_count().set(error_count);

        log::info!("Fetched {} nominators.", data.nominators.len());
        let mut success_count = 0;
        let mut error_count = 0;
        for (index, nominator) in data.nominators.iter().enumerate() {
            let save_result = postgres
                .save_onekv_nominator(nominator, CONFIG.dn.candidate_history_record_count as i64)
                .await;
            match save_result {
                Ok(_) => {
                    success_count += 1;
                    log::info!(
                        "Persisted nominator {} of {} :: {}.",
                        index + 1,
                        data.nominators.len(),
                        nominator,
                    );
                }
                Err(error) => {
                    error_count += 1;
                    log::error!("Error while persisting nominator {}:{:?}", nominator, error);
                }
            }
        }
        metrics::last_nominator_persist_success_count().set(success_count);
        metrics::last_nominator_persist_error_count().set(error_count);

        log::info!("DN update completed.");
        Ok(())
    }
}

#[async_trait(?Send)]
impl Service for OneKVUpdater {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.onekv_updater_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!(
            "DN updater has started with {} seconds refresh wait period.",
            CONFIG.dn.refresh_seconds
        );
        let postgres =
            PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
        loop {
            log::info!("Update DN data.");
            metrics::last_run_timestamp_ms().set(chrono::Utc::now().timestamp_millis());
            if let Err(error) = self.update_data(&postgres).await {
                metrics::last_candidate_count().set(0);
                metrics::last_candidate_list_fetch_success_status().set(0);
                metrics::last_candidate_persist_success_count().set(0);
                metrics::last_candidate_persist_error_count().set(0);
                log::error!("DN update has failed: {:?}", error);
            }
            log::info!("Sleep for {} seconds.", CONFIG.dn.refresh_seconds);
            tokio::time::sleep(std::time::Duration::from_secs(CONFIG.dn.refresh_seconds)).await;
        }
    }
}
