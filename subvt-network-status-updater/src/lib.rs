//! Updates the Redis database after every block with network status data.
//! Subscribes to the new blocks using the Substrate client in `subvt-substrate-client`.
#![warn(clippy::disallowed_types)]
use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use redis::Pipeline;
use std::sync::{Arc, Mutex};
use subvt_config::Config;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::metadata::get_metadata_era_duration_millis;
use subvt_types::{substrate::BlockHeader, subvt::NetworkStatus};

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct NetworkStatusUpdater {
    last_network_status: Mutex<NetworkStatus>,
}

impl NetworkStatusUpdater {
    /// Updates the Redis database with the given network status data.
    async fn update_redis(status: &NetworkStatus) -> anyhow::Result<()> {
        let redis_client = redis::Client::open(CONFIG.redis.url.as_str())?;
        let mut redis_connection = redis_client
            .get_multiplexed_async_connection()
            .await
            .context(format!(
                "Cannot connect to Redis at URL {}.",
                CONFIG.redis.url
            ))?;
        let status_json_string = serde_json::to_string(status)?;
        let mut redis_cmd_pipeline = Pipeline::new();
        redis_cmd_pipeline
            .cmd("SET")
            .arg(format!("subvt:{}:network_status", CONFIG.substrate.chain))
            .arg(status_json_string)
            .cmd("PUBLISH")
            .arg(format!(
                "subvt:{}:network_status:publish:best_block_number",
                CONFIG.substrate.chain
            ))
            .arg(status.best_block_number)
            .query_async::<()>(&mut redis_connection)
            .await
            .context("Error while publishing Redis pub/sub event.")?;
        Ok(())
    }

    async fn fetch_and_update_network_status(
        &self,
        relay_client: &SubstrateClient,
        asset_hub_client: &SubstrateClient,
        relay_best_block_header: &BlockHeader,
    ) -> anyhow::Result<NetworkStatus> {
        let last_status = {
            let guard = self.last_network_status.lock().unwrap();
            guard.clone()
        };
        // relay best block number
        let relay_best_block_number = relay_best_block_header
            .get_number()
            .context("Error while extracting relay best block number.")?;
        let relay_best_block_hash = relay_client
            .get_block_hash(relay_best_block_number)
            .await
            .context("Error while fetching relay best block hash.")?;
        log::debug!("Relay best block #{relay_best_block_number} hash {relay_best_block_hash}.",);
        // relay finalized block number & hash
        let relay_finalized_block_hash = relay_client
            .get_finalized_block_hash()
            .await
            .context("Error while fetching relay finalized block hash.")?;
        let relay_finalized_block_header = relay_client
            .get_block_header(relay_finalized_block_hash.as_str())
            .await
            .context("Error while fetching relay finalized block header.")?;
        let finalized_block_number = relay_finalized_block_header
            .get_number()
            .context("Error while extracting relay finalized block number.")?;
        log::debug!(
            "Relay finalized block #{finalized_block_number} hash {relay_finalized_block_hash}.",
        );
        // asset hub finalized block number & hash
        let asset_hub_finalized_block_hash = asset_hub_client
            .get_finalized_block_hash()
            .await
            .context("Error while fetching asset hub finalized block hash.")?;
        log::debug!("Asset hub finalized block hash {asset_hub_finalized_block_hash}.",);

        let era = asset_hub_client
            .get_active_era(
                asset_hub_finalized_block_hash.as_str(),
                &relay_client.metadata,
            )
            .await
            .context("Error while getting current era from asset hub.")?;
        let epoch = relay_client
            .get_current_epoch(&era, relay_finalized_block_hash.as_str())
            .await
            .context("Error while getting current epoch from relay.")?;
        let epoch_remaining = epoch.get_end_date_time() - Utc::now();
        log::debug!(
            "Epoch {} start {} end {}. {} days {} hours {} minutes {} seconds.",
            epoch.index,
            epoch.get_start_date_time().format("%d-%m-%Y %H:%M:%S"),
            epoch.get_end_date_time().format("%d-%m-%Y %H:%M:%S"),
            epoch_remaining.num_days(),
            epoch_remaining.num_hours() - epoch_remaining.num_days() * 24,
            epoch_remaining.num_minutes() - epoch_remaining.num_hours() * 60,
            epoch_remaining.num_seconds() - epoch_remaining.num_minutes() * 60,
        );
        // active and inactive validator counts
        let active_validator_account_ids = asset_hub_client
            .get_active_validator_account_ids(asset_hub_finalized_block_hash.as_str())
            .await
            .context("Error while getting active validator addresses from asset hub.")?;
        // number of validators
        let total_validator_count = asset_hub_client
            .get_total_validator_count(asset_hub_finalized_block_hash.as_str())
            .await
            .context("Error while getting total validator count from asset hub.")?;
        let active_validator_count = active_validator_account_ids.len() as u32;
        let inactive_validator_count = total_validator_count - active_validator_count;
        // era index & time
        let era_remaining = era.get_end_date_time() - Utc::now();
        log::debug!(
            "Era {} start {} end {}. {} days {} hours {} minutes {} seconds.",
            era.index,
            era.get_start_date_time().format("%d-%m-%Y %H:%M:%S"),
            era.get_end_date_time().format("%d-%m-%Y %H:%M:%S"),
            era_remaining.num_days(),
            era_remaining.num_hours() - era_remaining.num_days() * 24,
            era_remaining.num_minutes() - era_remaining.num_hours() * 60,
            era_remaining.num_seconds() - era_remaining.num_minutes() * 60,
        );

        let (
            last_era_total_reward,
            total_stake,
            return_rate_per_million,
            min_stake,
            max_stake,
            average_stake,
            median_stake,
        ) = if last_status.active_era.index == era.index {
            log::debug!("Era hasn't changed.");
            (
                last_status.last_era_total_reward,
                last_status.total_stake,
                last_status.return_rate_per_million,
                last_status.min_stake,
                last_status.max_stake,
                last_status.average_stake,
                last_status.median_stake,
            )
        } else {
            let last_era_total_reward = asset_hub_client
                .get_era_total_validator_reward(
                    era.index - 1,
                    asset_hub_finalized_block_hash.as_str(),
                )
                .await
                .context("Error while getting last era's total validator reward.")?;
            // era stakers
            let era_stakers = asset_hub_client
                .get_era_stakers(&era, asset_hub_finalized_block_hash.as_str())
                .await
                .context("Error while getting last era's active stakers.")?;
            let total_stake = era_stakers.total_stake();
            let era_duration_seconds = get_metadata_era_duration_millis(
                &relay_client.metadata,
                &asset_hub_client.metadata,
            )? / 1000;
            let year_seconds = (365 * 24 + 6) * 60 * 60;
            let eras_per_year = (year_seconds / era_duration_seconds) as u128;
            let return_rate_per_million =
                (last_era_total_reward * eras_per_year * 1000000 / total_stake) as u32;
            (
                last_era_total_reward,
                era_stakers.total_stake(),
                return_rate_per_million,
                era_stakers.min_stake().1,
                era_stakers.max_stake().1,
                era_stakers.average_stake(),
                era_stakers.median_stake(),
            )
        };
        let last_era_total_reward_decimals: String = format!(
            "{}",
            last_era_total_reward % 10u128.pow(asset_hub_client.system_properties.token_decimals)
        )
        .chars()
        .take(4)
        .collect();
        log::debug!(
            "Last era total reward {}.{}{}.",
            last_era_total_reward as u64
                / 10u64.pow(asset_hub_client.system_properties.token_decimals),
            last_era_total_reward_decimals,
            asset_hub_client.system_properties.token_symbol
        );
        log::debug!(
            "Return rate per cent {} total stake {} min stake {} max stake {} average stake {} median stake {}.",
            return_rate_per_million / 10000,
            total_stake,
            min_stake,
            max_stake,
            average_stake,
            median_stake,
        );
        // era reward points so far
        let era_reward_points = asset_hub_client
            .get_era_reward_points(era.index, &asset_hub_finalized_block_hash)
            .await
            .context("Error while getting current era reward points.")?
            .total;
        log::debug!("{era_reward_points} total reward points so far.");
        // prepare data
        let network_status = NetworkStatus {
            finalized_block_number,
            finalized_block_hash: relay_finalized_block_hash,
            best_block_number: relay_best_block_number,
            best_block_hash: relay_best_block_hash,
            active_era: era,
            current_epoch: epoch,
            active_validator_count,
            inactive_validator_count,
            last_era_total_reward,
            total_stake,
            return_rate_per_million,
            min_stake,
            max_stake,
            average_stake,
            median_stake,
            era_reward_points,
        };
        // write to redis
        NetworkStatusUpdater::update_redis(&network_status).await?;
        log::debug!("Redis updated.");
        Ok(network_status)
    }
}

impl NetworkStatusUpdater {
    async fn on_new_relay_block(
        &self,
        relay_substrate_client: Arc<SubstrateClient>,
        asset_hub_substrate_client: Arc<SubstrateClient>,
        relay_best_block_header: BlockHeader,
    ) -> anyhow::Result<()> {
        if let Ok(relay_best_block_number) = relay_best_block_header.get_number() {
            metrics::target_best_block_number().set(relay_best_block_number as i64);
            log::info!("New relay best block #{relay_best_block_number}.");
        }
        let start = std::time::Instant::now();
        let update_result = self
            .fetch_and_update_network_status(
                &relay_substrate_client,
                &asset_hub_substrate_client,
                &relay_best_block_header,
            )
            .await;
        match update_result {
            Ok(network_status) => {
                log::info!(
                    "Processed relay best block #{}.",
                    network_status.best_block_number
                );
                metrics::processing_time_ms().observe(start.elapsed().as_millis() as f64);
                metrics::processed_best_block_number().set(network_status.best_block_number as i64);
                let mut last_network_status = self.last_network_status.lock().unwrap();
                *last_network_status = network_status;
                Ok(())
            }
            Err(error) => {
                log::error!("{error:?}");
                log::error!(
                    "Network status update failed for block #{}. Will try again with the next block.",
                    relay_best_block_header.get_number().unwrap_or(0),
                );
                Err(error)
            }
        }
    }
}

/// Service implementation.
#[async_trait(?Send)]
impl Service for NetworkStatusUpdater {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.network_status_updater_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        loop {
            let relay_substrate_client = Arc::new(
                SubstrateClient::new(
                    CONFIG.substrate.rpc_url.as_str(),
                    CONFIG.substrate.network_id,
                    CONFIG.substrate.connection_timeout_seconds,
                    CONFIG.substrate.request_timeout_seconds,
                )
                .await?,
            );
            let asset_hub_substrate_client = Arc::new(
                SubstrateClient::new(
                    CONFIG.substrate.asset_hub_rpc_url.as_str(),
                    CONFIG.substrate.network_id,
                    CONFIG.substrate.connection_timeout_seconds,
                    CONFIG.substrate.request_timeout_seconds,
                )
                .await?,
            );
            let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
            relay_substrate_client
                .subscribe_to_new_blocks(
                    CONFIG.substrate.request_timeout_seconds,
                    |best_block_header| async {
                        let error_cell = error_cell.clone();
                        if let Some(error) = error_cell.get() {
                            return Err(anyhow::anyhow!("{:?}", error));
                        }
                        let relay_substrate_client = relay_substrate_client.clone();
                        let asset_hub_substrate_client = asset_hub_substrate_client.clone();
                        tokio::spawn(async move {
                            if let Err(error) = self
                                .on_new_relay_block(
                                    relay_substrate_client,
                                    asset_hub_substrate_client,
                                    best_block_header,
                                )
                                .await
                            {
                                let _ = error_cell.set(error);
                            }
                        });
                        Ok(())
                    },
                )
                .await;
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            log::error!(
                "New block subscription exited. Will refresh connection and subscription after {delay_seconds} seconds.",
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
        }
    }
}
