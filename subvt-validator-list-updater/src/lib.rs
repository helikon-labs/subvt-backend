//! Updates the Redis database with the complete validator list after every block.
//! Subscribes to the new blocks using the Substrate client in `subvt-substrate-client`.
#![warn(clippy::disallowed_types)]
use anyhow::Context;
use async_lock::RwLock;
use async_trait::async_trait;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use redis::Pipeline;
use rustc_hash::{FxHashSet as HashSet, FxHasher};
use std::hash::{Hash, Hasher};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::{BlockHeader, Era};
use subvt_types::subvt::{ValidatorDetails, ValidatorSummary};

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
    static ref IS_BUSY: AtomicBool = AtomicBool::new(false);
}

#[derive(Default)]
pub struct ValidatorListUpdater;

impl ValidatorListUpdater {
    async fn update_redis(
        active_era: &Era,
        finalized_block_number: u64,
        finalized_block_hash: &str,
        finalized_block_timestamp: u64,
        validators: &[ValidatorDetails],
    ) -> anyhow::Result<()> {
        // get redis connection
        let redis_client = redis::Client::open(CONFIG.redis.url.as_str())?;
        let mut redis_connection = redis_client
            .get_multiplexed_async_connection()
            .await
            .context(format!(
                "Cannot connect to Redis at URL {}.",
                CONFIG.redis.url
            ))?;
        let prefix = format!(
            "subvt:{}:validators:{finalized_block_number}",
            CONFIG.substrate.chain,
        );
        let active_account_ids: HashSet<String> = validators
            .iter()
            .filter_map(|validator| {
                if validator.is_active {
                    Some(validator.account.id.to_string())
                } else {
                    None
                }
            })
            .collect();
        let inactive_account_ids: HashSet<String> = validators
            .iter()
            .filter_map(|validator| {
                if !validator.is_active {
                    Some(validator.account.id.to_string())
                } else {
                    None
                }
            })
            .collect();
        let mut redis_cmd_pipeline = Pipeline::new();
        redis_cmd_pipeline.cmd("MSET");
        log::info!("Prepare validator details JSON entries.");
        // set validator details
        for validator in validators {
            let validator_prefix = format!(
                "{}:{}:validator:{}",
                prefix,
                if validator.is_active {
                    "active"
                } else {
                    "inactive"
                },
                validator.account.id
            );
            // calculate hash
            let hash = {
                let mut hasher = FxHasher::default();
                validator.hash(&mut hasher);
                hasher.finish()
            };
            // calculate summary hash
            let summary_hash = {
                let mut hasher = FxHasher::default();
                ValidatorSummary::from(validator).hash(&mut hasher);
                hasher.finish()
            };
            let validator_json_string = serde_json::to_string(validator)?;
            redis_cmd_pipeline
                .arg(format!("{validator_prefix}:hash"))
                .arg(hash)
                .arg(format!("{validator_prefix}:summary_hash"))
                .arg(summary_hash)
                .arg(validator_prefix)
                .arg(validator_json_string);
        }
        // set active and inactive account id sets
        redis_cmd_pipeline
            .cmd("SADD")
            .arg(format!("{prefix}:active:{}", "account_id_set"))
            .arg(active_account_ids);
        redis_cmd_pipeline
            .cmd("SADD")
            .arg(format!("{prefix}:inactive:{}", "account_id_set"))
            .arg(inactive_account_ids);
        redis_cmd_pipeline.cmd("MSET");
        // set finalized block number
        redis_cmd_pipeline
            .arg(format!(
                "subvt:{}:validators:finalized_block_number",
                CONFIG.substrate.chain
            ))
            .arg(finalized_block_number);
        redis_cmd_pipeline
            .arg(format!(
                "subvt:{}:validators:finalized_block_hash",
                CONFIG.substrate.chain
            ))
            .arg(finalized_block_hash);
        redis_cmd_pipeline
            .arg(format!(
                "subvt:{}:validators:finalized_block_timestamp",
                CONFIG.substrate.chain
            ))
            .arg(finalized_block_timestamp);
        // set era
        redis_cmd_pipeline
            .arg(format!("{prefix}:active_era"))
            .arg(serde_json::to_string(active_era)?);
        // publish event
        redis_cmd_pipeline
            .cmd("PUBLISH")
            .arg(format!(
                "subvt:{}:validators:publish:finalized_block_number",
                CONFIG.substrate.chain
            ))
            .arg(finalized_block_number);
        log::info!("Write to Redis.");
        () = redis_cmd_pipeline
            .query_async(&mut redis_connection)
            .await
            .context("Error while setting Redis validators.")?;
        Ok(())
    }

    async fn clear_history(processed_block_numbers: &Arc<RwLock<Vec<u64>>>) -> anyhow::Result<()> {
        log::info!("Clean redundant Redis history.");
        let redis_client = redis::Client::open(CONFIG.redis.url.as_str())?;
        let mut redis_connection = redis_client
            .get_multiplexed_async_connection()
            .await
            .context(format!(
                "Cannot connect to Redis at URL {}.",
                CONFIG.redis.url
            ))?;
        let mut redis_cmd_pipeline = Pipeline::new();
        let mut processed_block_numbers = processed_block_numbers.write().await;
        let to_delete: Vec<u64> = processed_block_numbers
            .iter()
            .cloned()
            .take(
                processed_block_numbers
                    .len()
                    .saturating_sub(CONFIG.validator_list_updater.history_record_depth as usize),
            )
            .collect();
        log::info!("Delete from history: {:?}", to_delete);
        for delete in to_delete {
            let keys: Vec<String> = redis::cmd("KEYS")
                .arg(format!(
                    "subvt:{}:validators:{delete}:*",
                    CONFIG.substrate.chain,
                ))
                .query_async(&mut redis_connection)
                .await?;
            log::info!("Delete {} records for block #{}.", keys.len(), delete);
            for key in keys {
                redis_cmd_pipeline.cmd("DEL").arg(key);
            }
            processed_block_numbers.remove(0);
        }
        () = redis_cmd_pipeline
            .query_async(&mut redis_connection)
            .await
            .context("Error while setting Redis validators.")?;
        Ok(())
    }

    async fn fetch_and_update_validator_list(
        client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        processed_block_numbers: &Arc<RwLock<Vec<u64>>>,
        finalized_block_header: &BlockHeader,
    ) -> anyhow::Result<Vec<ValidatorDetails>> {
        let finalized_block_number = finalized_block_header
            .get_number()
            .context("Error while extracting finalized block number.")?;
        log::info!("Process new finalized block #{}.", finalized_block_number);
        let finalized_block_hash = client
            .get_block_hash(finalized_block_number)
            .await
            .context("Error while fetching finalized block hash.")?;
        let finalized_block_timestamp = client.get_block_timestamp(&finalized_block_hash).await?;
        let active_era = client.get_active_era(&finalized_block_hash).await?;
        // validator account ids
        let mut validators = client
            .get_all_validators(finalized_block_hash.as_str(), &active_era)
            .await
            .context("Error while getting validators.")?;
        // enrich data with data from the relational database
        log::info!("Get RDB content.");
        for validator in validators.iter_mut() {
            let db_validator_info = postgres
                .get_validator_info(
                    &finalized_block_hash,
                    &validator.account.id,
                    validator.is_active,
                    active_era.index,
                )
                .await?;
            validator.account.discovered_at = db_validator_info.discovered_at;
            validator.slash_count = db_validator_info.slash_count;
            validator.offline_offence_count = db_validator_info.offline_offence_count;
            validator.active_era_count = db_validator_info.active_era_count;
            validator.inactive_era_count = db_validator_info.inactive_era_count;
            validator
                .unclaimed_era_indices
                .clone_from(&db_validator_info.unclaimed_era_indices);
            validator.blocks_authored = db_validator_info.blocks_authored;
            validator.reward_points = db_validator_info.reward_points;
            validator.heartbeat_received = db_validator_info.heartbeat_received;
            validator.onekv_candidate_record_id = db_validator_info.onekv_candidate_record_id;
            validator.onekv_rank = db_validator_info.onekv_rank;
            validator.onekv_location = db_validator_info.onekv_location;
            validator.onekv_is_valid = db_validator_info.onekv_is_valid;
            validator.onekv_offline_since = db_validator_info.onekv_offline_since;
        }
        log::info!("Got RDB content. Update Redis.");
        let start = std::time::Instant::now();
        ValidatorListUpdater::update_redis(
            &active_era,
            finalized_block_number,
            &finalized_block_hash,
            finalized_block_timestamp,
            &validators,
        )
        .await?;
        let elapsed = start.elapsed();
        log::info!("Redis updated. Took {} ms.", elapsed.as_millis());
        {
            let mut processed_block_numbers = processed_block_numbers.write().await;
            processed_block_numbers.push(finalized_block_number);
        }
        ValidatorListUpdater::clear_history(processed_block_numbers).await?;
        // update Redis processed block numbers
        {
            let processed_block_numbers = processed_block_numbers.read().await;
            ValidatorListUpdater::store_processed_block_numbers(&processed_block_numbers).await?;
        }
        Ok(validators)
    }

    async fn store_processed_block_numbers(processed_block_numbers: &[u64]) -> anyhow::Result<()> {
        let redis_client = redis::Client::open(CONFIG.redis.url.as_str())?;
        let mut redis_connection = redis_client
            .get_multiplexed_async_connection()
            .await
            .context(format!(
                "Cannot connect to Redis at URL {}.",
                CONFIG.redis.url
            ))?;
        () = redis::cmd("SET")
            .arg(&[
                format!(
                    "subvt:{}:validators:processed_block_numbers",
                    CONFIG.substrate.chain
                ),
                serde_json::to_string(processed_block_numbers)?,
            ])
            .query_async(&mut redis_connection)
            .await?;
        Ok(())
    }

    async fn fetch_processed_block_numbers() -> anyhow::Result<Vec<u64>> {
        let redis_client = redis::Client::open(CONFIG.redis.url.as_str())?;
        let mut redis_connection = redis_client
            .get_multiplexed_async_connection()
            .await
            .context(format!(
                "Cannot connect to Redis at URL {}.",
                CONFIG.redis.url
            ))?;
        let key = format!(
            "subvt:{}:validators:processed_block_numbers",
            CONFIG.substrate.chain
        );
        let key_exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut redis_connection)
            .await?;
        if key_exists {
            Ok(serde_json::from_str(
                &redis::cmd("GET")
                    .arg(&key)
                    .query_async::<_, String>(&mut redis_connection)
                    .await?,
            )?)
        } else {
            Ok(vec![])
        }
    }
}

#[async_trait(?Send)]
impl Service for ValidatorListUpdater {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.validator_list_updater_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        loop {
            if IS_BUSY.load(Ordering::SeqCst) {
                let delay_seconds = CONFIG.common.recovery_retry_seconds;
                log::warn!(
                    "Busy processing past state. Hold re-instantiation for {} seconds.",
                    delay_seconds
                );
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
                continue;
            }
            let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
            let postgres = Arc::new(
                PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?,
            );
            let substrate_client = Arc::new(SubstrateClient::new(&CONFIG).await?);
            let processed_block_numbers: Arc<RwLock<Vec<u64>>> = Arc::new(RwLock::new(
                ValidatorListUpdater::fetch_processed_block_numbers().await?,
            ));
            substrate_client.subscribe_to_finalized_blocks(
                CONFIG.substrate.request_timeout_seconds,
                |finalized_block_header| async {
                    let error_cell = error_cell.clone();
                    if let Some(error) = error_cell.get() {
                        return Err(anyhow::anyhow!("{:?}", error));
                    }
                    let finalized_block_number = match finalized_block_header.get_number() {
                        Ok(block_number) => block_number,
                        Err(error) => {
                            log::error!("Cannot get block number for header: {:?}", finalized_block_header);
                            return Err(anyhow::anyhow!("{:?}", error));
                        }
                    };
                    metrics::target_finalized_block_number().set(finalized_block_number as i64);
                    if IS_BUSY.load(Ordering::SeqCst) {
                        log::debug!("Busy processing a past block. Skip block #{}.", finalized_block_number);
                        return Ok(());
                    }
                    IS_BUSY.store(true, Ordering::SeqCst);
                    let processed_block_numbers = processed_block_numbers.clone();
                    let substrate_client = Arc::clone(&substrate_client);
                    let postgres = postgres.clone();
                    tokio::spawn(async move {
                        let start = std::time::Instant::now();
                        let update_result = ValidatorListUpdater::fetch_and_update_validator_list(
                            &substrate_client,
                            &postgres,
                            &processed_block_numbers,
                            &finalized_block_header,
                        ).await;
                        if let Err(error) = update_result {
                            log::error!("{:?}", error);
                            log::error!(
                                "Validator list update failed for block #{}. Will try again with the next block.",
                                finalized_block_header.get_number().unwrap_or(0),
                            );
                            let _ = error_cell.set(error);
                        } else {
                            metrics::processing_time_ms().observe(start.elapsed().as_millis() as f64);
                            metrics::processed_finalized_block_number().set(finalized_block_number as i64);
                        }
                        IS_BUSY.store(false, Ordering::SeqCst);
                    });
                    Ok(())
            }).await;
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            log::error!(
                "New block subscription exited. Will refresh connection and subscription after {} seconds.",
                delay_seconds
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
        }
    }
}
