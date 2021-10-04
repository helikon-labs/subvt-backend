//! Updates the Redis database after every block with inactive validator list data.
//! Subscribes to the new blocks using the Substrate client in `subvt-substrate-client`.

use anyhow::Context;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug, error};
use redis::Pipeline;
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use subvt_config::Config;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::BlockHeader;
use subvt_types::subvt::InactiveValidator;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct InactiveValidatorListUpdater;

impl InactiveValidatorListUpdater {
    fn update_redis(
        finalized_block_number: u64,
        finalized_block_hash: String,
        validators: &[InactiveValidator],
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
        let prefix = format!("subvt:{}:inactive_validators", CONFIG.substrate.chain);
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
            "subvt:{}:inactive_validators:validator:*",
            CONFIG.substrate.chain
        ));
        redis_cmd_pipeline.cmd("MSET");
        for validator in validators {
            let prefix = format!(
                "subvt:{}:inactive_validators:validator:{}",
                CONFIG.substrate.chain,
                validator.account.id.to_string()
            );
            // calculate hash
            let hash = {
                let mut hasher = DefaultHasher::new();
                validator.hash(&mut hasher);
                hasher.finish()
            };
            let validator_json_string = serde_json::to_string(validator)?;
            redis_cmd_pipeline
                .arg(format!("{}:hash", prefix))
                .arg(hash)
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
            .context("Error while setting Redis inactive validators.")?;
        Ok(())
    }

    async fn fetch_and_update_inactive_validator_list(
        client: &SubstrateClient,
        finalized_block_header: &BlockHeader,
    ) -> anyhow::Result<Vec<InactiveValidator>> {
        let finalized_block_number = finalized_block_header
            .get_number()
            .context("Error while extracting finalized block number.")?;
        let finalized_block_hash = client
            .get_block_hash(finalized_block_number)
            .await
            .context("Error while fetching finalized block hash.")?;
        // validator addresses
        let inactive_validators = client
            .get_all_inactive_validators(finalized_block_hash.as_str())
            .await
            .context("Error while getting inactive validators.")?;
        debug!("Fetched {} inactive validators.", inactive_validators.len());
        let start = std::time::Instant::now();
        InactiveValidatorListUpdater::update_redis(
            finalized_block_number,
            finalized_block_hash,
            &inactive_validators,
        )?;
        let elapsed = start.elapsed();
        debug!("Redis updated. Took {} ms.", elapsed.as_millis());
        Ok(inactive_validators)
    }
}

#[async_trait]
impl Service for InactiveValidatorListUpdater {
    async fn run(&'static self) -> anyhow::Result<()> {
        loop {
            let substrate_client = Arc::new(SubstrateClient::new(&CONFIG).await?);
            let is_busy = Arc::new(AtomicBool::new(false));
            substrate_client.subscribe_to_finalized_blocks(|finalized_block_header| {
                let substrate_client = Arc::clone(&substrate_client);
                let is_busy = Arc::clone(&is_busy);
                if is_busy.load(Ordering::Relaxed) {
                    debug!("Busy updating. Skip finalized block #{}.", finalized_block_header.get_number().unwrap_or(0));
                    return;
                }
                debug!("Process new finalized block #{}.", finalized_block_header.get_number().unwrap_or(0));
                is_busy.store(true, Ordering::Relaxed);
                tokio::spawn(async move {
                    let update_result = InactiveValidatorListUpdater::fetch_and_update_inactive_validator_list(
                        &substrate_client,
                        &finalized_block_header,
                    ).await;
                    match update_result {
                        Ok(_) => {
                            debug!("Update successful.");
                        }
                        Err(error) => {
                            error!("{:?}", error);
                            error!(
                                "Inactive validator list update failed for block #{}. Will try again with the next block.",
                                finalized_block_header.get_number().unwrap_or(0),
                            );
                        }
                    }
                    is_busy.store(false, Ordering::Relaxed);
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
