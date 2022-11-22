//! Indexes historical block data into the PostreSQL database instance.
#![warn(clippy::disallowed_types)]
use crate::event::process_event;
use async_lock::Mutex;
use async_trait::async_trait;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use rustc_hash::FxHashMap as HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::error::DecodeError;
use subvt_types::substrate::event::SubstrateEvent;
use subvt_types::substrate::metadata::MetadataVersion;
use subvt_types::substrate::ValidityAttestation;
use subvt_types::{
    crypto::AccountId,
    substrate::{Era, EraStakers, ValidatorStake},
};

mod event;
mod extrinsic;
mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
    static ref IS_BUSY: AtomicBool = AtomicBool::new(false);
}

static PARA_VALIDATOR_GROUP_SIZE: u32 = 5;

#[derive(Default)]
pub struct BlockProcessor;

#[derive(Default)]
struct RuntimeInformation {
    pub era_index: u32,
    pub epoch_index: u64,
}

impl BlockProcessor {
    async fn persist_era_validators_and_stakers(
        &self,
        substrate_client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        era: &Era,
        block_hash: &str,
        active_validator_account_ids: &[AccountId],
        era_stakers: &EraStakers,
    ) -> anyhow::Result<()> {
        log::info!("Persist era #{} validators.", era.index);
        let all_validator_account_ids = substrate_client
            .get_all_validator_account_ids(block_hash)
            .await?;
        let bonded_account_id_map = substrate_client
            .get_bonded_account_id_map(&all_validator_account_ids, block_hash)
            .await?;
        let validator_stake_map = {
            let mut validator_stake_map: HashMap<AccountId, ValidatorStake> = HashMap::default();
            for validator_stake in &era_stakers.stakers {
                validator_stake_map.insert(validator_stake.account.id, validator_stake.clone());
            }
            validator_stake_map
        };
        let validator_prefs_map = substrate_client
            .get_era_validator_prefs(era.index, block_hash)
            .await?;
        postgres
            .save_era_validators(
                era.index,
                active_validator_account_ids,
                &all_validator_account_ids,
                &bonded_account_id_map,
                &validator_stake_map,
                &validator_prefs_map,
            )
            .await?;
        postgres.save_era_stakers(era_stakers).await?;
        log::info!("Persisted era #{} validators and stakers.", era.index);
        Ok(())
    }

    async fn persist_era_reward_points(
        &self,
        substrate_client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        block_hash: &str,
        era_index: u32,
    ) -> anyhow::Result<()> {
        if !postgres.era_exists(era_index).await? {
            log::info!(
                "Era {} does not exist in the database. Cannot persist reward points.",
                era_index
            );
            return Ok(());
        }
        let era_reward_points = substrate_client
            .get_era_reward_points(era_index, block_hash)
            .await?;
        postgres
            .update_era_reward_points(era_index, era_reward_points.total)
            .await?;
        let mut era_reward_points_map: HashMap<AccountId, u32> = HashMap::default();
        era_reward_points
            .individual
            .iter()
            .for_each(|(account_id_32, reward_points)| {
                let account_id_bytes: &[u8; 32] = account_id_32.as_ref();
                let account_id = AccountId::new(*account_id_bytes);
                era_reward_points_map.insert(account_id, *reward_points);
            });
        postgres
            .update_era_validator_reward_points(era_index, era_reward_points_map)
            .await?;
        log::info!("Era {} rewards persisted.", era_index);
        Ok(())
    }

    async fn process_block(
        &self,
        substrate_client: &mut SubstrateClient,
        runtime_information: &Arc<RwLock<RuntimeInformation>>,
        postgres: &PostgreSQLNetworkStorage,
        block_number: u64,
        persist_era_reward_points: bool,
    ) -> anyhow::Result<()> {
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        let block_header = substrate_client.get_block_header(&block_hash).await?;
        let maybe_validator_index = block_header.get_validator_index();
        let runtime_upgrade_info = substrate_client
            .get_last_runtime_upgrade_info(&block_hash)
            .await?;
        // check metadata version
        if substrate_client
            .metadata
            .last_runtime_upgrade_info
            .spec_version
            != runtime_upgrade_info.spec_version
        {
            log::info!(
                "Different runtime version #{} than client's #{}. Will reset metadata.",
                runtime_upgrade_info.spec_version,
                substrate_client
                    .metadata
                    .last_runtime_upgrade_info
                    .spec_version
            );
            substrate_client
                .set_metadata_at_block(block_number, &block_hash)
                .await?;
            log::info!(
                "Runtime {} metadata fetched.",
                substrate_client
                    .metadata
                    .last_runtime_upgrade_info
                    .spec_version
            );
            //substrate_client.metadata.log_all_calls();
            //substrate_client.metadata.log_all_events();
        }
        let metadata_version = match substrate_client.metadata.version {
            MetadataVersion::V12 => 12,
            MetadataVersion::V13 => 13,
            MetadataVersion::V14 => 14,
        } as i16;
        let (last_era_index, last_epoch_index) = {
            let runtime_information = runtime_information.read().unwrap();
            (
                runtime_information.era_index,
                runtime_information.epoch_index,
            )
        };
        let active_era = substrate_client.get_active_era(&block_hash).await?;
        let current_epoch = substrate_client.get_current_epoch(&block_hash).await?;
        let active_validator_account_ids = substrate_client
            .get_active_validator_account_ids(&block_hash)
            .await?;
        if last_epoch_index != current_epoch.index || last_era_index != active_era.index {
            let era_stakers = substrate_client
                .get_era_stakers(&active_era, true, &block_hash)
                .await?;
            if last_epoch_index != current_epoch.index {
                log::info!("New epoch. Persist epoch, and persist era if it doesn't exist.");
                let total_stake = substrate_client
                    .get_era_total_stake(active_era.index, &block_hash)
                    .await?;
                postgres
                    .save_era(&active_era, total_stake, &era_stakers)
                    .await?;
                postgres
                    .save_epoch(&current_epoch, active_era.index)
                    .await?;
                // save session para validators
                if let Some(para_validator_indices) = substrate_client
                    .get_paras_active_validator_indices(&block_hash)
                    .await?
                {
                    log::info!(
                        "Persist {} session para validators.",
                        para_validator_indices.len()
                    );
                    let para_validator_groups = substrate_client
                        .get_para_validator_groups(&block_hash)
                        .await?;
                    for (group_index, group_para_validator_indices) in
                        para_validator_groups.iter().enumerate()
                    {
                        for group_para_validator_index in group_para_validator_indices {
                            let active_validator_index =
                                para_validator_indices[*group_para_validator_index as usize];
                            let active_validator_account_id =
                                active_validator_account_ids[active_validator_index as usize];
                            postgres
                                .save_session_para_validator(
                                    active_era.index,
                                    current_epoch.index,
                                    &active_validator_account_id,
                                    active_validator_index,
                                    group_index as u32,
                                    *group_para_validator_index,
                                )
                                .await?;
                        }
                    }
                } else {
                    log::info!("Parachains not active at this block height.");
                }
            }
            if last_era_index != active_era.index {
                let era_stakers = substrate_client
                    .get_era_stakers(&active_era, true, &block_hash)
                    .await?;
                self.persist_era_validators_and_stakers(
                    substrate_client,
                    postgres,
                    &active_era,
                    block_hash.as_str(),
                    &active_validator_account_ids,
                    &era_stakers,
                )
                .await?;
                // update last era
                let last_era_total_validator_reward = substrate_client
                    .get_era_total_validator_reward(active_era.index - 1, &block_hash)
                    .await?;
                postgres
                    .update_era_total_validator_reward(
                        active_era.index - 1,
                        last_era_total_validator_reward,
                    )
                    .await?;
                self.persist_era_reward_points(
                    substrate_client,
                    postgres,
                    &block_hash,
                    active_era.index - 1,
                )
                .await?;
            }
        }
        if persist_era_reward_points {
            self.persist_era_reward_points(
                substrate_client,
                postgres,
                &block_hash,
                active_era.index,
            )
            .await?;
        }
        {
            let mut runtime_information = runtime_information.write().unwrap();
            runtime_information.era_index = active_era.index;
            runtime_information.epoch_index = current_epoch.index;
        }
        let event_results = substrate_client.get_block_events(&block_hash).await?;
        log::info!(
            "Got {} events for block #{}.",
            event_results.len(),
            block_number
        );
        let extrinsic_results = substrate_client.get_block_extrinsics(&block_hash).await?;
        log::info!(
            "Got {} extrinsics for block #{}.",
            extrinsic_results.len(),
            block_number
        );

        let block_timestamp = substrate_client.get_block_timestamp(&block_hash).await?;
        let maybe_author_account_id = if let Some(validator_index) = maybe_validator_index {
            active_validator_account_ids
                .get(validator_index)
                .map(|a| a.to_owned())
        } else {
            None
        };
        let runtime_version = substrate_client
            .metadata
            .last_runtime_upgrade_info
            .spec_version as i16;
        postgres
            .save_finalized_block(
                &block_hash,
                &block_header,
                block_timestamp,
                maybe_author_account_id,
                (active_era.index, current_epoch.index as u32),
                (metadata_version, runtime_version),
            )
            .await?;
        // process/persist events
        let mut extrinsic_event_map: HashMap<u32, Vec<(usize, SubstrateEvent)>> =
            HashMap::default();
        for (index, event_result) in event_results.iter().enumerate() {
            match event_result {
                Ok(event) => {
                    if let Some(extrinsic_index) = event.get_extrinsic_index() {
                        if let Some(extrinsic_events) =
                            extrinsic_event_map.get_mut(&extrinsic_index)
                        {
                            extrinsic_events.push((index, event.clone()))
                        } else {
                            extrinsic_event_map
                                .insert(extrinsic_index, vec![(index, event.clone())]);
                        }
                    }
                    if let Err(error) = process_event(
                        substrate_client,
                        postgres,
                        current_epoch.index,
                        &block_hash,
                        block_number,
                        block_timestamp,
                        index,
                        event,
                    )
                    .await
                    {
                        let error_log = format!(
                            "Error while processing event #{} of block #{}: {:?}",
                            index, block_number, error,
                        );
                        log::error!("{}", error_log);
                        metrics::event_process_error_count().inc();
                        postgres
                            .save_event_process_error_log(
                                &block_hash,
                                block_number,
                                index,
                                "process",
                                &error_log,
                            )
                            .await?;
                    }
                }
                Err(decode_error) => match decode_error {
                    DecodeError::Error(error_log) => {
                        metrics::event_process_error_count().inc();
                        postgres
                            .save_event_process_error_log(
                                &block_hash,
                                block_number,
                                index,
                                "decode",
                                error_log,
                            )
                            .await?;
                        panic!("Panic due to event decode error: {error_log:?}");
                    }
                },
            }
        }
        // persist extrinsics
        for (index, extrinsic_result) in extrinsic_results.iter().enumerate() {
            match extrinsic_result {
                Ok(extrinsic) => {
                    // check events for batch & batch_all
                    if let Err(error) = self
                        .process_extrinsic(
                            substrate_client,
                            postgres,
                            block_hash.clone(),
                            block_number,
                            &active_validator_account_ids,
                            index,
                            false,
                            &None,
                            None,
                            None,
                            extrinsic_event_map
                                .get_mut(&(index as u32))
                                .unwrap_or(&mut vec![]),
                            false,
                            extrinsic,
                        )
                        .await
                    {
                        let error_log = format!(
                            "Error while processing extrinsic #{} of block #{}: {:?}",
                            index, block_number, error,
                        );
                        log::error!("{}", error_log);
                        metrics::extrinsic_process_error_count().inc();
                        postgres
                            .save_extrinsic_process_error_log(
                                &block_hash,
                                block_number,
                                index,
                                "process",
                                &error_log,
                            )
                            .await?;
                    }
                }
                Err(decode_error) => match decode_error {
                    DecodeError::Error(error_log) => {
                        metrics::extrinsic_process_error_count().inc();
                        postgres
                            .save_extrinsic_process_error_log(
                                &block_hash,
                                block_number,
                                index,
                                "decode",
                                error_log,
                            )
                            .await?;
                    }
                },
            }
        }
        // para core assignments
        if let Some(para_core_assignments) = substrate_client
            .get_para_core_assignments(&block_hash)
            .await?
        {
            for para_core_assignment in &para_core_assignments {
                postgres
                    .save_para_core_assignment(&block_hash, para_core_assignment)
                    .await?;
            }
            log::debug!(
                "Processed {} para core scheduler assignments.",
                para_core_assignments.len()
            );
        }
        // para votes
        if let Some(votes) = substrate_client.get_para_votes(&block_hash).await? {
            let session_index: u32 = votes.session;
            let mut total_vote_count = 0;
            for backing in &votes.backing_validators_per_candidate {
                let para_id: u32 = backing.0.descriptor.para_id.into();
                // get scheduled para validators from previous block
                total_vote_count += backing.1.len();
                let mut voted_para_validator_indices = vec![];
                for validator_vote in &backing.1 {
                    let para_validator_index = validator_vote.0 .0;
                    let is_explicit = match validator_vote.1 {
                        ValidityAttestation::Implicit(_) => false,
                        ValidityAttestation::Explicit(_) => true,
                    };
                    voted_para_validator_indices.push(para_validator_index);
                    postgres
                        .save_para_vote(
                            &block_hash,
                            session_index,
                            para_id,
                            para_validator_index,
                            Some(is_explicit),
                        )
                        .await?;
                }
                // save missing votes
                let para_validator_group_index = voted_para_validator_indices[0] / 5;
                let group_para_validator_indices: Vec<u32> = ((para_validator_group_index
                    * PARA_VALIDATOR_GROUP_SIZE)
                    ..(para_validator_group_index * PARA_VALIDATOR_GROUP_SIZE
                        + PARA_VALIDATOR_GROUP_SIZE))
                    .collect();
                for group_para_validator_index in group_para_validator_indices {
                    if !voted_para_validator_indices.contains(&group_para_validator_index) {
                        postgres
                            .save_para_vote(
                                &block_hash,
                                session_index,
                                para_id,
                                group_para_validator_index,
                                None,
                            )
                            .await?;
                    }
                }
            }
            log::debug!(
                "Processed {} para votes for {} paras.",
                total_vote_count,
                votes.backing_validators_per_candidate.len(),
            );
        }
        // notify
        postgres
            .notify_block_processed(block_number, block_hash)
            .await?;
        Ok(())
    }
}

/// Service implementation.
#[async_trait(?Send)]
impl Service for BlockProcessor {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.block_processor_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        loop {
            if IS_BUSY.load(Ordering::SeqCst) {
                let delay_seconds = CONFIG.common.recovery_retry_seconds;
                log::warn!(
                    "Busy processing past blocks. Hold re-instantiation for {} seconds.",
                    delay_seconds
                );
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
                continue;
            }
            let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
            let block_subscription_substrate_client = SubstrateClient::new(&CONFIG).await?;
            let block_processor_substrate_client =
                Arc::new(Mutex::new(SubstrateClient::new(&CONFIG).await?));
            let runtime_information = Arc::new(RwLock::new(RuntimeInformation::default()));
            let postgres = Arc::new(
                PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?,
            );
            // init extrinsic and event process error count metrics
            metrics::extrinsic_process_error_count()
                .set(postgres.get_extrinsic_process_error_log_count().await? as i64);
            metrics::event_process_error_count()
                .set(postgres.get_event_process_error_log_count().await? as i64);

            block_subscription_substrate_client.subscribe_to_finalized_blocks(
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
                        log::debug!("Busy processing past blocks. Skip block #{} for now.", finalized_block_number);
                        return Ok(());
                    }

                    let block_processor_substrate_client = block_processor_substrate_client.clone();
                    let runtime_information = runtime_information.clone();
                    let postgres = postgres.clone();
                    IS_BUSY.store(true, Ordering::SeqCst);
                    tokio::spawn(async move {
                        let mut block_processor_substrate_client = block_processor_substrate_client.lock().await;
                        let processed_block_height = match postgres.get_processed_block_height().await {
                            Ok(processed_block_height) => processed_block_height,
                            Err(error) => {
                                log::error!("Cannot get processed block height from the database: {:?}", error);
                                let _ = error_cell.set(error);
                                IS_BUSY.store(false, Ordering::SeqCst);
                                return;
                            }
                        };
                        if processed_block_height < (finalized_block_number - 1) {
                            let mut block_number = std::cmp::max(
                                processed_block_height,
                                CONFIG.block_processor.start_block_number
                            );
                            while block_number <= finalized_block_number {
                                log::info!(
                                    "Process block #{}. Target #{}.",
                                    block_number,
                                    finalized_block_number
                                );
                                let start = std::time::Instant::now();
                                let process_result = self.process_block(
                                    &mut block_processor_substrate_client,
                                    &runtime_information,
                                    &postgres,
                                    block_number,
                                    false,
                                ).await;
                                metrics::block_processing_time_ms().observe(start.elapsed().as_millis() as f64);
                                match process_result {
                                    Ok(_) => {
                                        metrics::processed_finalized_block_number().set(block_number as i64);
                                        block_number += 1
                                    },
                                    Err(error) => {
                                        log::error!("{:?}", error);
                                        log::error!(
                                            "History block processing failed for block #{}.",
                                            block_number,
                                        );
                                        let _ = error_cell.set(error);
                                        break;
                                    }
                                }
                            }
                        } else {
                            // update current era reward points every 3 minutes
                            let blocks_per_3_minutes = 3 * 60 * 1000
                                / block_processor_substrate_client
                                .metadata
                                .constants
                                .expected_block_time_millis;
                            log::info!("Process block #{}.", finalized_block_number);
                            let start = std::time::Instant::now();
                            let update_result = self.process_block(
                                &mut block_processor_substrate_client,
                                &runtime_information,
                                &postgres,
                                finalized_block_number,
                                finalized_block_number % blocks_per_3_minutes == 0,
                            ).await;
                            metrics::block_processing_time_ms().observe(start.elapsed().as_millis() as f64);
                            match update_result {
                                Ok(_) => {
                                    metrics::processed_finalized_block_number().set(finalized_block_number as i64);
                                },
                                Err(error) => {
                                    log::error!("{:?}", error);
                                    log::error!(
                                        "Block processing failed for finalized block #{}. Will try again with the next block.",
                                        finalized_block_header.get_number().unwrap_or(0),
                                    );
                                    let _ = error_cell.set(error);
                                }
                            }
                        }
                        IS_BUSY.store(false, Ordering::SeqCst);
                    });
                    Ok(())
            }).await;
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            log::error!(
                "Finalized block subscription exited. Will refresh connection and subscription after {} seconds.",
                delay_seconds
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
        }
    }
}
