use crate::NotificationGenerator;
use anyhow::Context;
use log::{debug, error, warn};
use redis::Connection;
use std::collections::{HashMap, HashSet};
use subvt_config::Config;
use subvt_types::subvt::ValidatorDetails;

fn populate_validator_map(
    connection: &mut Connection,
    prefix: &str,
    active_validator_account_ids: &HashSet<String>,
    all_validator_account_ids: &HashSet<String>,
    validator_map: &mut HashMap<String, ValidatorDetails>,
) -> anyhow::Result<()> {
    let all_keys: Vec<String> = all_validator_account_ids
        .iter()
        .map(|account_id| {
            format!(
                "{}:{}:validator:{}",
                prefix,
                if active_validator_account_ids.contains(account_id) {
                    "active"
                } else {
                    "inactive"
                },
                account_id
            )
        })
        .collect();
    let validator_json_strings: Vec<String> = redis::cmd("MGET")
        .arg(&all_keys)
        .query(connection)
        .context("Can't read validator json string from Redis.")
        .unwrap();
    println!("Got all JSON {}.", validator_json_strings.len());
    for validator_json_string in validator_json_strings {
        let validator: ValidatorDetails = serde_json::from_str(&validator_json_string).unwrap();
        validator_map.insert(validator.account.id.to_string(), validator);
    }
    println!("Complete {}.", validator_map.len());
    Ok(())
}

impl NotificationGenerator {
    pub fn process_validator_list_updates(config: &Config) {
        loop {
            // intialize Redis
            let redis_client = redis::Client::open(config.redis.url.as_str())
                .context(format!(
                    "Cannot connect to Redis at URL {}.",
                    config.redis.url
                ))
                .unwrap();
            let mut pub_sub_connection = redis_client.get_connection().unwrap();
            let mut pub_sub = pub_sub_connection.as_pubsub();
            let mut data_connection = redis_client.get_connection().unwrap();
            let _ = pub_sub
                .subscribe(format!(
                    "subvt:{}:validators:publish:finalized_block_number",
                    config.substrate.chain
                ))
                .unwrap();
            // keep this to avoid duplicate block processing
            let mut last_finalized_block_number = 0;
            // keep track of validators
            let mut validator_map: HashMap<String, ValidatorDetails> = HashMap::new();

            let error: anyhow::Error = loop {
                let message = pub_sub.get_message();
                if let Err(error) = message {
                    break error.into();
                }
                let payload = message.unwrap().get_payload();
                if let Err(error) = payload {
                    break error.into();
                }
                let finalized_block_number: u64 = payload.unwrap();
                if last_finalized_block_number == finalized_block_number {
                    warn!(
                        "Skip duplicate finalized block #{}.",
                        finalized_block_number
                    );
                    continue;
                }
                debug!(
                    "New finalized block from validator list updater #{}.",
                    finalized_block_number
                );
                let prefix = format!(
                    "subvt:{}:validators:{}",
                    config.substrate.chain, finalized_block_number
                );
                let active_validator_account_ids: HashSet<String> = match redis::cmd("SMEMBERS")
                    .arg(format!("{}:active:account_id_set", prefix))
                    .query(&mut data_connection)
                {
                    Ok(set) => set,
                    Err(error) => break error.into(),
                };
                let inactive_validator_account_ids: HashSet<String> = match redis::cmd("SMEMBERS")
                    .arg(format!("{}:inactive:account_id_set", prefix))
                    .query(&mut data_connection)
                {
                    Ok(set) => set,
                    Err(error) => break error.into(),
                };
                let all_validator_account_ids: HashSet<String> = active_validator_account_ids
                    .union(&inactive_validator_account_ids)
                    .cloned()
                    .collect();
                if validator_map.is_empty() {
                    if let Err(error) = populate_validator_map(
                        &mut data_connection,
                        &prefix,
                        &active_validator_account_ids,
                        &all_validator_account_ids,
                        &mut validator_map,
                    ) {
                        break error;
                    }
                    continue;
                }
                last_finalized_block_number = finalized_block_number;
            };
            let delay_seconds = config.common.recovery_retry_seconds;
            error!(
                "Error while processing validator list: {:?}. Sleep for {} seconds, then continue.",
                error, delay_seconds,
            );
            std::thread::sleep(std::time::Duration::from_secs(delay_seconds));
        }
    }
}
