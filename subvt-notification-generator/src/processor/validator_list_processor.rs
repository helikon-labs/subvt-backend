//! Checks validator changes for notifications. Validator list in Redis gets updated by
//! `subvt-validator-list-updater`, and the update is notified using the Redis PUBLISH function.
//! Keeps a copy of the validator list in heap memory (vector) to track changes.

use crate::NotificationGenerator;
use anyhow::Context;
use chrono::Utc;
use log::{debug, error, info, warn};
use redis::Connection;
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::app_event::{OneKVRankChange, OneKVValidityChange};
use subvt_types::substrate::Era;
use subvt_types::{
    app::app_event,
    app::NotificationTypeCode,
    crypto::AccountId,
    substrate::{Balance, Nomination},
    subvt::ValidatorDetails,
};

/// Does the initial population of the cached validator map.
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
    /// Runs after each notification from the validator list updater for each validator,
    /// checks for changes in the validator and persists notifications where a rule requires it.
    async fn check_validator_changes(
        config: &Config,
        (app_postgres, network_postgres): (&PostgreSQLAppStorage, &PostgreSQLNetworkStorage),
        substrate_client: &Arc<SubstrateClient>,
        redis_connection: &mut Connection,
        redis_prefix: &str,
        finalized_block_number: u64,
        last: &ValidatorDetails,
    ) -> anyhow::Result<Option<ValidatorDetails>> {
        let address = &last.account.address;
        // last hash
        let hash = {
            let mut hasher = DefaultHasher::new();
            last.hash(&mut hasher);
            hasher.finish()
        };
        // current hash
        let db_hash: u64 = redis::cmd("GET")
            .arg(format!("{}:hash", redis_prefix))
            .query(redis_connection)
            .context("Can't read validator hash from Redis.")?;
        // return if there's no change in the validator's details
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
            .map(|nomination| &nomination.stash_account.id)
            .cloned()
            .collect();
        let last_nominator_ids: HashSet<AccountId> = last
            .nominations
            .iter()
            .map(|nomination| &nomination.stash_account.id)
            .cloned()
            .collect();
        let new_nominator_ids = &current_nominator_ids - &last_nominator_ids;
        let lost_nominator_ids = &last_nominator_ids - &current_nominator_ids;
        let renominator_ids = &current_nominator_ids - &new_nominator_ids;
        let mut current_nomination_map: HashMap<&AccountId, &Nomination> = HashMap::new();
        for nomination in &current.nominations {
            current_nomination_map.insert(&nomination.stash_account.id, nomination);
        }
        let mut last_nomination_map: HashMap<&AccountId, &Nomination> = HashMap::new();
        for nomination in &last.nominations {
            last_nomination_map.insert(&nomination.stash_account.id, nomination);
        }
        // new nominations
        for new_nominator_id in new_nominator_ids {
            let new_nomination = *current_nomination_map.get(&new_nominator_id).unwrap();
            debug!(
                "New nomination for {} :: {} :: {}",
                address,
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
            let event = app_event::NewNomination {
                validator_account_id: current.account.id.clone(),
                discovered_block_number: finalized_block_number,
                nominator_stash_account_id: new_nomination.stash_account.id.clone(),
                active_amount: new_nomination.stake.active_amount,
                total_amount: new_nomination.stake.total_amount,
                nominee_count: new_nomination.target_account_ids.len() as u64,
            };
            for rule in rules {
                if let Some(min_param) = rule.parameters.get(0) {
                    if let Ok(min_amount) = min_param.value.parse::<Balance>() {
                        if new_nomination.stake.active_amount < min_amount {
                            continue;
                        }
                    }
                }
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    substrate_client,
                    &[rule],
                    finalized_block_number,
                    &current.account.id,
                    Some(&event),
                )
                .await?;
            }
            network_postgres.save_new_nomination_event(&event).await?;
        }
        // lost nominations
        for lost_nominator_id in lost_nominator_ids {
            let lost_nomination = *last_nomination_map.get(&lost_nominator_id).unwrap();
            // create app event
            debug!(
                "Lost nomination for {} :: {} :: {}",
                address,
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
            let event = app_event::LostNomination {
                validator_account_id: current.account.id.clone(),
                discovered_block_number: finalized_block_number,
                nominator_stash_account_id: lost_nomination.stash_account.id.clone(),
                active_amount: lost_nomination.stake.active_amount,
                total_amount: lost_nomination.stake.total_amount,
                nominee_count: lost_nomination.target_account_ids.len() as u64,
            };
            for rule in rules {
                if let Some(min_param) = rule.parameters.get(0) {
                    if let Ok(min_amount) = min_param.value.parse::<Balance>() {
                        if lost_nomination.stake.active_amount < min_amount {
                            continue;
                        }
                    }
                }
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    substrate_client,
                    &[rule],
                    finalized_block_number,
                    &current.account.id,
                    Some(&event),
                )
                .await?;
            }
            network_postgres.save_lost_nomination_event(&event).await?;
        }
        // nomination amount changes
        for renominator_id in renominator_ids {
            let current_nomination = *current_nomination_map.get(&renominator_id).unwrap();
            let prev_nomination = *last_nomination_map.get(&renominator_id).unwrap();
            if current_nomination.stake.active_amount != prev_nomination.stake.active_amount {
                // create app event
                debug!(
                    "Changed nomination for {} :: {} :: {} -> {}",
                    address,
                    renominator_id.to_ss58_check(),
                    prev_nomination.stake.active_amount,
                    current_nomination.stake.active_amount,
                );
                let rules = app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::ChainValidatorNominationAmountChange.to_string(),
                        config.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                let event = app_event::NominationAmountChange {
                    validator_account_id: current.account.id.clone(),
                    discovered_block_number: finalized_block_number,
                    nominator_stash_account_id: current_nomination.stash_account.id.clone(),
                    prev_active_amount: prev_nomination.stake.active_amount,
                    prev_total_amount: prev_nomination.stake.total_amount,
                    prev_nominee_count: prev_nomination.target_account_ids.len() as u64,
                    active_amount: current_nomination.stake.active_amount,
                    total_amount: current_nomination.stake.total_amount,
                    nominee_count: current_nomination.target_account_ids.len() as u64,
                };
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    substrate_client,
                    &rules,
                    finalized_block_number,
                    &current.account.id,
                    Some(&event),
                )
                .await?;
                network_postgres
                    .save_nomination_amount_change_event(&event)
                    .await?;
            }
        }
        // check (in)active next session
        if current.active_next_session != last.active_next_session {
            if current.active_next_session {
                debug!("Active next session: {}", current.account.address,);
                let rules = app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::ChainValidatorActiveNextSession.to_string(),
                        config.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    substrate_client,
                    &rules,
                    finalized_block_number,
                    &current.account.id,
                    None::<&()>,
                )
                .await?;
                network_postgres
                    .save_active_next_session_event(&current.account.id, finalized_block_number)
                    .await?;
            } else {
                debug!("Inactive next session: {}", current.account.address,);
                let rules = app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::ChainValidatorInactiveNextSession.to_string(),
                        config.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    substrate_client,
                    &rules,
                    finalized_block_number,
                    &current.account.id,
                    None::<&()>,
                )
                .await?;
                network_postgres
                    .save_inactive_next_session_event(&current.account.id, finalized_block_number)
                    .await?;
            }
        }
        // check (in)active now
        if current.is_active != last.is_active {
            if current.is_active {
                debug!("Now active: {}", current.account.address);
                let rules = app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::ChainValidatorActive.to_string(),
                        config.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    substrate_client,
                    &rules,
                    finalized_block_number,
                    &current.account.id,
                    None::<&()>,
                )
                .await?;
                network_postgres
                    .save_active_event(&current.account.id, finalized_block_number)
                    .await?;
            } else {
                debug!("Now inactive: {}", current.account.id.to_ss58_check());
                let rules = app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::ChainValidatorInactive.to_string(),
                        config.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    substrate_client,
                    &rules,
                    finalized_block_number,
                    &current.account.id,
                    None::<&()>,
                )
                .await?;
                network_postgres
                    .save_inactive_event(&current.account.id, finalized_block_number)
                    .await?;
            }
        }
        // check 1kv rank and validity
        if current.onekv_candidate_record_id.is_some()
            && (current.onekv_candidate_record_id == last.onekv_candidate_record_id)
        {
            if current.onekv_rank != last.onekv_rank {
                debug!(
                    "1KV rank of {} changed from {} to {}.",
                    current.account.address,
                    last.onekv_rank.unwrap(),
                    current.onekv_rank.unwrap(),
                );
                let rules = app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::OneKVValidatorRankChange.to_string(),
                        config.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    substrate_client,
                    &rules,
                    finalized_block_number,
                    &current.account.id,
                    Some(&OneKVRankChange {
                        validator_account_id: current.account.id.clone(),
                        prev_rank: last.onekv_rank.unwrap(),
                        current_rank: current.onekv_rank.unwrap(),
                    }),
                )
                .await?;
                network_postgres
                    .save_onekv_rank_change_event(
                        &current.account.id,
                        last.onekv_rank.unwrap(),
                        current.onekv_rank.unwrap(),
                    )
                    .await?;
            }
            // check validity
            if current.onekv_is_valid != last.onekv_is_valid {
                debug!(
                    "1KV validity of {} changed from {} to {}.",
                    current.account.address,
                    last.onekv_is_valid.unwrap(),
                    current.onekv_is_valid.unwrap(),
                );
                let rules = app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::OneKVValidatorValidityChange.to_string(),
                        config.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                let validity_items = network_postgres
                    .get_onekv_candidate_validity_items(current.onekv_candidate_record_id.unwrap())
                    .await?;
                NotificationGenerator::generate_notifications(
                    config,
                    app_postgres,
                    substrate_client,
                    &rules,
                    finalized_block_number,
                    &current.account.id,
                    Some(&OneKVValidityChange {
                        validator_account_id: current.account.id.clone(),
                        is_valid: current.onekv_is_valid.unwrap(),
                        validity_items,
                    }),
                )
                .await?;
                network_postgres
                    .save_onekv_validity_change_event(
                        &current.account.id,
                        current.onekv_is_valid.unwrap(),
                    )
                    .await?;
            }
        }
        Ok(Some(current))
    }

    /// Called after each validator list update PUBLISH event.
    async fn process(
        config: &Config,
        (app_postgres, network_postgres): (&PostgreSQLAppStorage, &PostgreSQLNetworkStorage),
        substrate_client: &Arc<SubstrateClient>,
        redis_connection: &mut Connection,
        validator_map: &mut HashMap<String, ValidatorDetails>,
        finalized_block_number: u64,
        last_active_era_index: &AtomicU32,
    ) -> anyhow::Result<()> {
        info!(
            "Process new update from validator list updater. Block #{}.",
            finalized_block_number
        );
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
            // first run
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
                network_postgres
                    .save_new_validator_event(&account_id, finalized_block_number)
                    .await?;
                // add to validator map
                let validator_prefix = format!(
                    "{}:{}:validator:{}",
                    prefix,
                    if active_validator_account_ids.contains(added_id) {
                        "active"
                    } else {
                        "inactive"
                    },
                    added_id
                );
                let validator = {
                    let db_validator_json: String = redis::cmd("GET")
                        .arg(validator_prefix)
                        .query(redis_connection)
                        .context("Can't read validator JSON from Redis.")?;
                    serde_json::from_str::<ValidatorDetails>(&db_validator_json)?
                };
                validator_map.insert(added_id.clone(), validator);
            }
            for removed_id in &removed_validator_ids {
                let account_id = AccountId::from_str(removed_id)?;
                info!("Remove validator: {}", account_id.to_ss58_check());
                network_postgres
                    .save_removed_validator_event(&account_id, finalized_block_number)
                    .await?;
                validator_map.remove(removed_id);
            }
            debug!("Checking for changes in existing validators.");
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
                    (app_postgres, network_postgres),
                    substrate_client,
                    redis_connection,
                    &validator_prefix,
                    finalized_block_number,
                    validator_map.get(validator_id).unwrap(),
                )
                .await?
                {
                    validator_map.insert(validator_id.clone(), updated);
                }
            }
            // check era change & unclaimed payouts
            let db_active_era_json: String = redis::cmd("GET")
                .arg(format!("{}:active_era", prefix))
                .query(redis_connection)
                .context("Can't read active era JSON from Redis.")?;
            let active_era: Era = serde_json::from_str(&db_active_era_json)?;
            let era_start = active_era.get_start_date_time();
            let era_elapsed = Utc::now() - era_start;
            if era_elapsed.num_hours()
                >= config
                    .notification_generator
                    .unclaimed_payout_check_delay_hours as i64
                && last_active_era_index.load(Ordering::SeqCst) != active_era.index
            {
                if !network_postgres
                    .notification_generator_has_processed_era(active_era.index)
                    .await?
                {
                    debug!("Process era #{} for unclaimed payouts.", active_era.index);
                    for validator in validator_map.values() {
                        if !validator.unclaimed_era_indices.is_empty() {
                            let rules = app_postgres
                                .get_notification_rules_for_validator(
                                    &NotificationTypeCode::ChainValidatorUnclaimedPayout
                                        .to_string(),
                                    config.substrate.network_id,
                                    &validator.account.id,
                                )
                                .await?;
                            // generate notifications
                            NotificationGenerator::generate_notifications(
                                config,
                                app_postgres,
                                substrate_client,
                                &rules,
                                finalized_block_number,
                                &validator.account.id,
                                Some(&validator.unclaimed_era_indices),
                            )
                            .await?;
                        }
                    }
                    network_postgres
                        .save_notification_generator_processed_era(active_era.index)
                        .await?;
                }
                // and add the era index to processed era indices
                last_active_era_index.store(active_era.index, Ordering::SeqCst);
            }
        }
        Ok(())
    }

    pub async fn process_validator_list_updates(
        config: &Config,
        substrate_client: Arc<SubstrateClient>,
    ) {
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
            let last_active_era_index = AtomicU32::new(0);

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
                if let Err(error) = NotificationGenerator::process(
                    config,
                    (&app_postgres, &network_postgres),
                    &substrate_client,
                    &mut data_connection,
                    &mut validator_map,
                    finalized_block_number,
                    &last_active_era_index,
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
