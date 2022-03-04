//! Updates the Redis database after every block with network status data.
//! Subscribes to the new blocks using the Substrate client in `subvt-substrate-client`.

use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use lazy_static::lazy_static;
use log::{debug, error};
use redis::Pipeline;
use std::sync::{Arc, Mutex};
use subvt_config::Config;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::{substrate::BlockHeader, subvt::NetworkStatus};

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct NetworkStatusUpdater {
    last_network_status: Mutex<NetworkStatus>,
}

impl NetworkStatusUpdater {
    /// Updates the Redis database with the given network status data.
    fn update_redis(status: &NetworkStatus) -> anyhow::Result<()> {
        let redis_client = redis::Client::open(CONFIG.redis.url.as_str())?;
        let mut redis_connection = redis_client.get_connection().context(format!(
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
            .query(&mut redis_connection)
            .context("Error while publishing Redis pub/sub event.")?;
        Ok(())
    }

    async fn fetch_and_update_network_status(
        &self,
        client: &SubstrateClient,
        best_block_header: &BlockHeader,
    ) -> anyhow::Result<NetworkStatus> {
        let last_status = {
            let guard = self.last_network_status.lock().unwrap();
            guard.clone()
        };
        // best block number
        let best_block_number = best_block_header
            .get_number()
            .context("Error while extracting best block number.")?;
        let best_block_hash = client
            .get_block_hash(best_block_number)
            .await
            .context("Error while fetching best block hash.")?;
        debug!(
            "Best block #{} hash {}.",
            best_block_number, best_block_hash
        );
        // finalized block number & hash
        let finalized_block_hash = client
            .get_finalized_block_hash()
            .await
            .context("Error while fetching finalized block hash.")?;
        let finalized_block_header = client
            .get_block_header(finalized_block_hash.as_str())
            .await
            .context("Error while fetching finalized block header.")?;
        let finalized_block_number = finalized_block_header
            .get_number()
            .context("Error while extracting finalized block number.")?;
        debug!(
            "Finalized block #{} hash {}.",
            finalized_block_number, finalized_block_hash
        );
        // epoch index & time
        let epoch = client
            .get_current_epoch(best_block_hash.as_str())
            .await
            .context("Error while getting current epoch.")?;
        let epoch_remaining = epoch.get_end_date_time() - Utc::now();
        debug!(
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
        let active_validator_account_ids = client
            .get_active_validator_account_ids(best_block_hash.as_str())
            .await
            .context("Error while getting active validator addresses.")?;
        // number of validators
        let total_validator_count = client
            .get_total_validator_count(best_block_hash.as_str())
            .await
            .context("Error while getting total validator count.")?;
        let active_validator_count = active_validator_account_ids.len() as u32;
        let inactive_validator_count = total_validator_count - active_validator_count;
        // era index & time
        let era = client
            .get_active_era(best_block_hash.as_str())
            .await
            .context("Error while getting active era.")?;
        let era_remaining = era.get_end_date_time() - Utc::now();
        debug!(
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
            debug!("Era hasn't changed.");
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
            let last_era_total_reward = client
                .get_era_total_validator_reward(era.index - 1, best_block_hash.as_str())
                .await
                .context("Error while getting last era's total validator reward.")?;
            // era stakers
            let era_stakers = client
                .get_era_stakers(&era, true, best_block_hash.as_str())
                .await
                .context("Error while getting last era's active stakers.")?;
            let total_stake = era_stakers.total_stake();
            let era_duration_seconds = client.metadata.constants.era_duration_millis / 1000;
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
            (last_era_total_reward % 10u128.pow(client.system_properties.token_decimals))
        )
        .chars()
        .take(4)
        .collect();
        debug!(
            "Last era total reward {}.{}{}.",
            last_era_total_reward as u64 / 10u64.pow(client.system_properties.token_decimals),
            last_era_total_reward_decimals,
            client.system_properties.token_symbol
        );
        debug!(
        "Return rate per cent {} total stake {} min stake {} max stake {} average stake {} median stake {}.",
        return_rate_per_million / 10000,
        total_stake,
        min_stake,
        max_stake,
        average_stake,
        median_stake,
    );
        // era reward points so far
        let era_reward_points = client
            .get_era_reward_points(era.index, &best_block_hash)
            .await
            .context("Error while getting current era reward points.")?
            .total;
        debug!("{} total reward points so far.", era_reward_points);
        // prepare data
        let network_status = NetworkStatus {
            finalized_block_number,
            finalized_block_hash,
            best_block_number,
            best_block_hash,
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
        NetworkStatusUpdater::update_redis(&network_status)?;
        debug!("Redis updated.");
        Ok(network_status)
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
            let substrate_client = Arc::new(SubstrateClient::new(&CONFIG).await?);
            substrate_client.subscribe_to_new_blocks(|best_block_header| {
                let substrate_client = Arc::clone(&substrate_client);
                tokio::spawn(async move {
                    let update_result = self.fetch_and_update_network_status(
                        &substrate_client,
                        &best_block_header,
                    ).await;
                    match update_result {
                        Ok(network_status) => {
                            let mut last_network_status = self.last_network_status.lock().unwrap();
                            *last_network_status = network_status;
                        }
                        Err(error) => {
                            error!("{:?}", error);
                            error!(
                                "Network status update failed for block #{}. Will try again with the next block.",
                                best_block_header.get_number().unwrap_or(0),
                            );
                        }
                    }
                });
            }).await?;
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            error!(
                "New block subscription exited. Will refresh connection and subscription after {} seconds.",
                delay_seconds
            );
            std::thread::sleep(std::time::Duration::from_secs(delay_seconds));
        }
    }
}
