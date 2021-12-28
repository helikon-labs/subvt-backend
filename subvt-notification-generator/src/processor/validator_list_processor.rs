use crate::NotificationGenerator;
use anyhow::Context;
use log::{debug, error, info, warn};
use redis::Connection;
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::NotificationTypeCode;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::{Balance, Nomination};
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
    debug!(
        "Got JSON string for {} validators.",
        validator_json_strings.len()
    );
    for validator_json_string in validator_json_strings {
        let validator: ValidatorDetails = serde_json::from_str(&validator_json_string).unwrap();
        validator_map.insert(validator.account.id.to_string(), validator);
    }
    info!(
        "Populated validator map with {} validators.",
        validator_map.len()
    );
    Ok(())
}

impl NotificationGenerator {
    async fn check_validator_changes(
        config: &Config,
        app_postgres: &PostgreSQLAppStorage,
        _network_postgres: &PostgreSQLNetworkStorage,
        redis_connection: &mut Connection,
        redis_prefix: &str,
        last: &ValidatorDetails,
    ) -> anyhow::Result<Option<ValidatorDetails>> {
        let account_id = &last.account.id;
        let hash = {
            let mut hasher = DefaultHasher::new();
            last.hash(&mut hasher);
            hasher.finish()
        };
        let db_hash: u64 = redis::cmd("GET")
            .arg(format!("{}:hash", redis_prefix))
            .query(redis_connection)
            .context("Can't read validator hash from Redis.")?;
        if hash == db_hash {
            return Ok(None);
        }
        let current = {
            let db_validator_json: String = redis::cmd("GET")
                .arg(redis_prefix)
                .query(redis_connection)
                .context("Can't read validator JSON from Redis.")?;
            serde_json::from_str::<ValidatorDetails>(&db_validator_json)?
        };

        let current_nominator_ids: HashSet<AccountId> = current
            .nominations
            .iter()
            .map(|nomination| &nomination.stash_account_id)
            .cloned()
            .collect();
        let last_nominator_ids: HashSet<AccountId> = last
            .nominations
            .iter()
            .map(|nomination| &nomination.stash_account_id)
            .cloned()
            .collect();
        let new_nominator_ids = &current_nominator_ids - &last_nominator_ids;
        let lost_nominator_ids = &last_nominator_ids - &current_nominator_ids;
        let renominator_ids = &current_nominator_ids - &new_nominator_ids;
        let mut current_nomination_map: HashMap<&AccountId, &Nomination> = HashMap::new();
        for nomination in &current.nominations {
            current_nomination_map.insert(&nomination.stash_account_id, nomination);
        }
        let mut last_nomination_map: HashMap<&AccountId, &Nomination> = HashMap::new();
        for nomination in &last.nominations {
            last_nomination_map.insert(&nomination.stash_account_id, nomination);
        }
        // find added
        for new_nominator_id in new_nominator_ids {
            let new_nomination = *current_nomination_map.get(&new_nominator_id).unwrap();
            // create app event
            println!(
                "New nomination for {} :: {} :: {}",
                account_id.to_ss58_check(),
                new_nominator_id.to_ss58_check(),
                new_nomination.stake.active_amount,
            );
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorNewNomination.to_string(),
                    config.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            for rule in rules {
                let (param_type_id, param_value) = if let Some(min_param) = rule.parameters.get(0) {
                    if let Ok(min_amount) = min_param.value.parse::<Balance>() {
                        if new_nomination.stake.active_amount < min_amount {
                            continue;
                        } else {
                            (
                                Some(min_param.parameter_type_id),
                                Some(new_nomination.stake.active_amount.to_string()),
                            )
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    &None,
                    (None, None),
                    &[rule],
                    &current.account.id,
                    (param_type_id, param_value, None::<()>),
                )
                .await?;
            }
            /* PERSIST APP EVENT
            id
            validator account id
            block
            nominator stash account id
            active amount
            total amount
            created at
             */
        }
        // find removed
        for lost_nominator_id in lost_nominator_ids {
            let lost_nomination = *last_nomination_map.get(&lost_nominator_id).unwrap();
            // create app event
            println!(
                "Lost nomination for {} :: {} :: {}",
                account_id.to_ss58_check(),
                lost_nominator_id.to_ss58_check(),
                lost_nomination.stake.active_amount,
            );
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorLostNomination.to_string(),
                    config.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            for rule in rules {
                let (param_type_id, param_value) = if let Some(min_param) = rule.parameters.get(0) {
                    if let Ok(min_amount) = min_param.value.parse::<Balance>() {
                        if lost_nomination.stake.active_amount < min_amount {
                            continue;
                        } else {
                            (
                                Some(min_param.parameter_type_id),
                                Some(lost_nomination.stake.active_amount.to_string()),
                            )
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    &None,
                    (None, None),
                    &[rule],
                    &current.account.id,
                    (param_type_id, param_value, None::<()>),
                )
                .await?;
            }
            /* PERSIST APP EVENT
            id
            validator account id
            block
            nominator stash account id
            active amount
            total amount
            created at
             */
        }
        // find amount changed
        for renominator_id in renominator_ids {
            let current_nomination = *current_nomination_map.get(&renominator_id).unwrap();
            let last_nomination = *last_nomination_map.get(&renominator_id).unwrap();
            if current_nomination.stake.active_amount != last_nomination.stake.active_amount {
                // create app event
                println!(
                    "CHANGED nomination for {} :: {}  :: {} -> {:?}",
                    account_id.to_ss58_check(),
                    renominator_id.to_ss58_check(),
                    last_nomination.stake.active_amount,
                    current_nomination.stake,
                );
                let _rules = app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::ChainValidatorNominationAmountChange.to_string(),
                        config.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                /*
                id
                block hash
                validator account id
                nominator stash account id
                prev active amount
                prev total amount
                active amount
                total amount
                is processed
                created at
                 */
            }
        }

        // check active next session
        if current.active_next_session != last.active_next_session {
            if current.active_next_session {
                println!("active next");
                /*
                id
                block hash
                era index
                epoch index
                validator account id
                is processed
                created at
                 */
            } else {
                println!("inactive next");
                /*
                id
                block hash
                era index
                epoch index
                validator account id
                is processed
                created at
                 */
            }
        }
        // check active
        if current.is_active != last.is_active {
            if current.is_active {
                println!("active");
                /*
                id
                block hash
                era index
                epoch index
                validator account id
                is processed
                created at
                 */
            } else {
                println!("inactive");
                /*
                id
                block hash
                era index
                epoch index
                validator account id
                is processed
                created at
                 */
            }
        }
        // check 1kv
        if current.onekv_candidate_record_id.is_some() {
            // check score
            if current.onekv_rank != last.onekv_rank {
                println!("onekv rank changed");
            }
            // check validity
            if current.onekv_is_valid != last.onekv_is_valid {
                println!("onekv validity changed");
            }
        }

        Ok(Some(current))
    }

    async fn process(
        config: &Config,
        app_postgres: &PostgreSQLAppStorage,
        network_postgres: &PostgreSQLNetworkStorage,
        redis_connection: &mut Connection,
        validator_map: &mut HashMap<String, ValidatorDetails>,
        finalized_block_number: u64,
    ) -> anyhow::Result<()> {
        let prefix = format!(
            "subvt:{}:validators:{}",
            config.substrate.chain, finalized_block_number
        );
        let active_validator_account_ids: HashSet<String> = redis::cmd("SMEMBERS")
            .arg(format!("{}:active:account_id_set", prefix))
            .query(redis_connection)?;
        let inactive_validator_account_ids: HashSet<String> = redis::cmd("SMEMBERS")
            .arg(format!("{}:inactive:account_id_set", prefix))
            .query(redis_connection)?;
        let all_validator_account_ids: HashSet<String> = active_validator_account_ids
            .union(&inactive_validator_account_ids)
            .cloned()
            .collect();
        if validator_map.is_empty() {
            info!("Validator map is empty. Populate.");
            populate_validator_map(
                redis_connection,
                &prefix,
                &active_validator_account_ids,
                &all_validator_account_ids,
                validator_map,
            )?;
        } else {
            let prev_validator_account_ids: HashSet<String> =
                validator_map.keys().cloned().collect();
            let added_validator_ids = &all_validator_account_ids - &prev_validator_account_ids;
            let removed_validator_ids = &prev_validator_account_ids - &all_validator_account_ids;
            let remaining_validator_ids = &all_validator_account_ids - &added_validator_ids;
            for added_id in &added_validator_ids {
                let account_id = AccountId::from_str(added_id)?;
                info!("Persist new validator: {}", account_id.to_ss58_check());
                network_postgres.save_new_validator(&account_id).await?;
            }
            for removed_id in &removed_validator_ids {
                let account_id = AccountId::from_str(removed_id)?;
                info!("Remove validator: {}", account_id.to_ss58_check());
                network_postgres.save_removed_validator(&account_id).await?;
            }
            debug!("Checking for changes in the remaining validators.");
            for validator_id in &remaining_validator_ids {
                let validator_prefix = format!(
                    "{}:{}:validator:{}",
                    prefix,
                    if active_validator_account_ids.contains(validator_id) {
                        "active"
                    } else {
                        "inactive"
                    },
                    validator_id
                );
                if let Some(updated) = NotificationGenerator::check_validator_changes(
                    config,
                    app_postgres,
                    network_postgres,
                    redis_connection,
                    &validator_prefix,
                    validator_map.get(validator_id).unwrap(),
                )
                .await?
                {
                    validator_map.insert(validator_id.clone(), updated);
                }
            }
        }
        Ok(())
    }

    pub async fn process_validator_list_updates(config: &Config) {
        loop {
            // initialize Postgres connection
            let app_postgres = PostgreSQLAppStorage::new(config, config.get_app_postgres_url())
                .await
                .unwrap();
            let network_postgres =
                PostgreSQLNetworkStorage::new(config, config.get_network_postgres_url())
                    .await
                    .unwrap();
            // intialize Redis connection
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
                if let Err(error) = NotificationGenerator::process(
                    config,
                    &app_postgres,
                    &network_postgres,
                    &mut data_connection,
                    &mut validator_map,
                    finalized_block_number,
                )
                .await
                {
                    break error;
                }
                info!("Completed checks for block #{}.", finalized_block_number);
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