//! Updates the Redis database after every block with validator list data.
//! Subscribes to the new blocks using the Substrate client in `subvt-substrate-client`.

use anyhow::Context;
use async_lock::Mutex;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug, error};
use redis::Pipeline;
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::PostgreSQLStorage;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::BlockHeader;
use subvt_types::subvt::{ValidatorDetails, ValidatorSummary};

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct ValidatorListUpdater;

impl ValidatorListUpdater {
    fn update_redis(
        finalized_block_number: u64,
        finalized_block_hash: String,
        validators: &[ValidatorDetails],
    ) -> anyhow::Result<()> {
        // get redis connection
        let redis_client = redis::Client::open(CONFIG.redis.url.as_str())?;
        let mut redis_connection = redis_client.get_connection().context(format!(
            "Cannot connect to Redis at URL {}.",
            CONFIG.redis.url
        ))?;
        // prepare first command pipeline
        let mut redis_cmd_pipeline = Pipeline::new();
        // block number and hash
        let prefix = format!("subvt:{}:validators", CONFIG.substrate.chain);
        redis_cmd_pipeline
            .cmd("MSET")
            .arg(format!("{}:{}", prefix, "finalized_block_number"))
            .arg(finalized_block_number)
            .arg(format!("{}:{}", prefix, "finalized_block_hash"))
            .arg(finalized_block_hash.as_str());
        // validator address list
        redis_cmd_pipeline
            .cmd("DEL")
            .arg(format!("{}:{}", prefix, "addresses"));
        let addresses: HashSet<String> = validators
            .iter()
            .map(|validator| validator.account.id.to_string())
            .collect();
        redis_cmd_pipeline
            .cmd("SADD")
            .arg(format!("{}:{}", prefix, "addresses"))
            .arg(addresses);
        // each validator
        redis_cmd_pipeline.cmd("DEL").arg(format!(
            "subvt:{}:validators:validator:*",
            CONFIG.substrate.chain
        ));
        redis_cmd_pipeline.cmd("MSET");
        for validator in validators {
            let prefix = format!(
                "subvt:{}:validators:validator:{}",
                CONFIG.substrate.chain, validator.account.id
            );
            // calculate hash
            let hash = {
                let mut hasher = DefaultHasher::new();
                validator.hash(&mut hasher);
                hasher.finish()
            };
            let summary_hash = {
                let mut hasher = DefaultHasher::new();
                ValidatorSummary::from(validator).hash(&mut hasher);
                hasher.finish()
            };
            let validator_json_string = serde_json::to_string(validator)?;
            redis_cmd_pipeline
                .arg(format!("{}:hash", prefix))
                .arg(hash)
                .arg(format!("{}:summary_hash", prefix))
                .arg(summary_hash)
                .arg(prefix)
                .arg(validator_json_string);
        }
        // publish event
        redis_cmd_pipeline
            .cmd("PUBLISH")
            .arg(format!("{}:publish:finalized_block_number", prefix))
            .arg(finalized_block_number);
        redis_cmd_pipeline
            .query(&mut redis_connection)
            .context("Error while setting Redis validators.")?;
        Ok(())
    }

    async fn fetch_and_update_validator_list(
        client: &SubstrateClient,
        postgres: &PostgreSQLStorage,
        finalized_block_header: &BlockHeader,
    ) -> anyhow::Result<Vec<ValidatorDetails>> {
        let finalized_block_number = finalized_block_header
            .get_number()
            .context("Error while extracting finalized block number.")?;
        debug!("Process new finalized block #{}.", finalized_block_number);
        let finalized_block_hash = client
            .get_block_hash(finalized_block_number)
            .await
            .context("Error while fetching finalized block hash.")?;
        let active_era = client.get_active_era(&finalized_block_hash).await?;
        // validator addresses
        let mut validators = client
            .get_all_validators(finalized_block_hash.as_str())
            .await
            .context("Error while getting validators.")?;
        debug!("Fetched {} validators.", validators.len());
        // enrich data with data from the relational database
        debug!("Get database content.");
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
            validator.account.killed_at = db_validator_info.killed_at;
            validator.slash_count = db_validator_info.slash_count;
            validator.offline_offence_count = db_validator_info.offline_offence_count;
            validator.active_era_count = db_validator_info.active_era_count;
            validator.inactive_era_count = db_validator_info.inactive_era_count;
            validator.total_reward_points = db_validator_info.total_reward_points;
            validator.unclaimed_era_indices = db_validator_info.unclaimed_era_indices.clone();
            validator.is_enrolled_in_1kv = db_validator_info.is_enrolled_in_1kv;
            validator.blocks_authored = db_validator_info.blocks_authored;
            validator.reward_points = db_validator_info.reward_points;
            validator.heartbeat_received = db_validator_info.heartbeat_received;
        }
        debug!("Got database content.");
        let start = std::time::Instant::now();
        ValidatorListUpdater::update_redis(
            finalized_block_number,
            finalized_block_hash,
            &validators,
        )?;
        let elapsed = start.elapsed();
        debug!("Redis updated. Took {} ms.", elapsed.as_millis());
        Ok(validators)
    }
}

#[async_trait]
impl Service for ValidatorListUpdater {
    async fn run(&'static self) -> anyhow::Result<()> {
        loop {
            let postgres = Arc::new(PostgreSQLStorage::new(&CONFIG).await?);
            let substrate_client = Arc::new(SubstrateClient::new(&CONFIG).await?);
            let busy_lock = Arc::new(Mutex::new(()));
            substrate_client.subscribe_to_finalized_blocks(|finalized_block_header| {
                let substrate_client = Arc::clone(&substrate_client);
                let busy_lock = busy_lock.clone();
                let postgres = postgres.clone();
                tokio::spawn(async move {
                    let _lock = busy_lock.lock().await;
                    let update_result = ValidatorListUpdater::fetch_and_update_validator_list(
                        &substrate_client,
                        &postgres,
                        &finalized_block_header,
                    ).await;
                    match update_result {
                        Ok(_) => {
                            debug!("Update successful.");
                        }
                        Err(error) => {
                            error!("{:?}", error);
                            error!(
                                "Validator list update failed for block #{}. Will try again with the next block.",
                                finalized_block_header.get_number().unwrap_or(0),
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
