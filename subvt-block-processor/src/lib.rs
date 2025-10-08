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
use subvt_types::substrate::metadata::get_metadata_expected_block_time_millis;
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
    static ref RELAY_IS_BUSY: AtomicBool = AtomicBool::new(false);
    static ref ASSET_HUB_IS_BUSY: AtomicBool = AtomicBool::new(false);
}

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
                "Era {era_index} does not exist in the database. Cannot persist reward points.",
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
        log::info!("Era {era_index} rewards persisted.");
        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    async fn process_relay_block(
        &'static self,
        substrate_client: &mut SubstrateClient,
        asset_hub_substrate_client: &mut SubstrateClient,
        runtime_information: &Arc<RwLock<RuntimeInformation>>,
        postgres: &PostgreSQLNetworkStorage,
        block_number: u64,
    ) -> anyhow::Result<()> {
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        let block_header = substrate_client.get_block_header(&block_hash).await?;
        let block_timestamp = substrate_client.get_block_timestamp(&block_hash).await?;
        let maybe_validator_index = block_header.get_validator_index();
        let runtime_upgrade_info = substrate_client
            .get_last_runtime_upgrade_info(&block_hash)
            .await?;
        let asset_hub_block_hash = asset_hub_substrate_client
            .get_finalized_block_hash()
            .await?;
        let active_era = {
            asset_hub_substrate_client
                .get_active_era(&asset_hub_block_hash, &substrate_client.metadata)
                .await?
        };
        // check metadata version
        if substrate_client.last_runtime_upgrade_info.spec_version
            != runtime_upgrade_info.spec_version
        {
            log::info!(
                "RELAY Different runtime version #{} than client's #{}. Will reset metadata.",
                runtime_upgrade_info.spec_version,
                substrate_client.last_runtime_upgrade_info.spec_version
            );
            substrate_client
                .set_metadata_at_block(block_number, &block_hash)
                .await?;
            log::info!(
                "RELAY Runtime {} metadata fetched.",
                substrate_client.last_runtime_upgrade_info.spec_version
            );
        }
        let (last_era_index, last_epoch_index) = {
            let runtime_information = runtime_information.read().unwrap();
            (
                runtime_information.era_index,
                runtime_information.epoch_index,
            )
        };
        let active_validator_account_ids = substrate_client
            .get_active_validator_account_ids(&block_hash)
            .await?;
        let current_epoch = substrate_client
            .get_current_epoch(&active_era, &block_hash)
            .await?;
        if last_epoch_index != current_epoch.index || last_era_index != active_era.index {
            let era_stakers = asset_hub_substrate_client
                .get_era_stakers(&active_era, &asset_hub_block_hash)
                .await?;
            if last_epoch_index != current_epoch.index {
                log::info!("New epoch. Persist epoch, and persist era if it doesn't exist.");
                let total_stake = asset_hub_substrate_client
                    .get_era_total_stake(active_era.index, &asset_hub_block_hash)
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
                let era_stakers = asset_hub_substrate_client
                    .get_era_stakers(&active_era, &asset_hub_block_hash)
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
                let last_era_total_validator_reward = asset_hub_substrate_client
                    .get_era_total_validator_reward(active_era.index - 1, &asset_hub_block_hash)
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
        {
            let mut runtime_information = runtime_information.write().unwrap();
            runtime_information.era_index = active_era.index;
            runtime_information.epoch_index = current_epoch.index;
        }

        let maybe_author_account_id = if let Some(validator_index) = maybe_validator_index {
            active_validator_account_ids
                .get(validator_index)
                .map(|a| a.to_owned())
        } else {
            None
        };
        let runtime_version = substrate_client.last_runtime_upgrade_info.spec_version as i16;
        let current_epoch = substrate_client
            .get_current_epoch(&active_era, &block_hash)
            .await?;
        postgres
            .save_finalized_block(
                "relay",
                &block_hash,
                &block_header,
                block_timestamp,
                maybe_author_account_id,
                (active_era.index, current_epoch.index as u32),
                (14, runtime_version),
            )
            .await?;

        // para core assignments
        if let Ok(Some(para_core_assignments)) = substrate_client
            .get_para_core_assignments(&block_hash)
            .await
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
        } else if let Ok(Some(para_core_assignments)) = substrate_client
            .get_para_core_assignments_legacy(&block_hash)
            .await
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
        // para groups
        let para_groups = substrate_client
            .get_para_validator_groups(&block_hash)
            .await?;
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
                if let Some(first_para_validator_index) = voted_para_validator_indices.first() {
                    for para_group in &para_groups {
                        if para_group.contains(first_para_validator_index) {
                            for para_validator_index in para_group {
                                if !voted_para_validator_indices.contains(para_validator_index) {
                                    postgres
                                        .save_para_vote(
                                            &block_hash,
                                            session_index,
                                            para_id,
                                            *para_validator_index,
                                            None,
                                        )
                                        .await?;
                                }
                            }
                            break;
                        }
                    }
                }
            }
            // save disputes
            for statement_set in votes.disputes {
                for _statement in statement_set.statements {}
            }
            log::debug!(
                "Processed {} para votes for {} paras.",
                total_vote_count,
                votes.backing_validators_per_candidate.len(),
            );
        }
        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    async fn process_asset_hub_block(
        &'static self,
        substrate_client: &mut SubstrateClient,
        relay_substrate_client: &mut SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        block_number: u64,
        persist_era_reward_points: bool,
    ) -> anyhow::Result<()> {
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        let block_header = substrate_client.get_block_header(&block_hash).await?;
        let runtime_upgrade_info = substrate_client
            .get_last_runtime_upgrade_info(&block_hash)
            .await?;
        let relay_block_hash = relay_substrate_client.get_finalized_block_hash().await?;
        // check metadata version
        if substrate_client.last_runtime_upgrade_info.spec_version
            != runtime_upgrade_info.spec_version
        {
            log::info!(
                "ASSET_HUB Different runtime version #{} than client's #{}. Will reset metadata.",
                runtime_upgrade_info.spec_version,
                substrate_client.last_runtime_upgrade_info.spec_version
            );
            substrate_client
                .set_metadata_at_block(block_number, &block_hash)
                .await?;
            log::info!(
                "ASSET_HUB Runtime {} metadata fetched.",
                substrate_client.last_runtime_upgrade_info.spec_version
            );
        }
        let active_validator_account_ids = relay_substrate_client
            .get_active_validator_account_ids(&relay_block_hash)
            .await?;
        let active_era = substrate_client
            .get_active_era(&block_hash, &relay_substrate_client.metadata)
            .await?;
        let current_epoch = relay_substrate_client
            .get_current_epoch(&active_era, &relay_block_hash)
            .await?;
        if persist_era_reward_points {
            self.persist_era_reward_points(
                substrate_client,
                postgres,
                &block_hash,
                active_era.index,
            )
            .await?;
        }
        let event_results = relay_substrate_client
            .get_block_events(&relay_block_hash)
            .await?;
        log::info!(
            "ASSET_HUB Got {} events for block #{}.",
            event_results.len(),
            block_number
        );
        let extrinsic_results = substrate_client.get_block_extrinsics(&block_hash).await?;
        log::info!(
            "ASSET_HUB Got {} extrinsics for block #{}.",
            extrinsic_results.len(),
            block_number
        );

        let block_timestamp = substrate_client.get_block_timestamp(&block_hash).await?;
        let runtime_version = substrate_client.last_runtime_upgrade_info.spec_version as i16;
        postgres
            .save_finalized_block(
                "asset_hub",
                &block_hash,
                &block_header,
                block_timestamp,
                None,
                (active_era.index, current_epoch.index as u32),
                (14, runtime_version),
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
                        postgres,
                        &block_hash,
                        block_number,
                        block_timestamp,
                        index,
                        event,
                    )
                    .await
                    {
                        let error_log = format!(
                            "Error while processing event #{index} of block #{block_number}: {error:?}",
                        );
                        log::error!("{error_log}");
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
                            "Error while processing extrinsic #{index} of block #{block_number}: {error:?}",
                        );
                        log::error!("{error_log}");
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
                        panic!("Panic due to extrinsic decode error: {error_log}");
                    }
                },
            }
        }
        // notify
        postgres
            .notify_block_processed(block_number, &block_hash)
            .await?;
        Ok(())
    }
}

impl BlockProcessor {
    async fn subscribe_relay_chain(&'static self) -> anyhow::Result<()> {
        loop {
            if RELAY_IS_BUSY.load(Ordering::SeqCst) {
                let delay_seconds = CONFIG.common.recovery_retry_seconds;
                log::warn!("RELAY Busy processing past blocks. Hold re-instantiation for {delay_seconds} seconds.");
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
                continue;
            }
            let relay_error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
            let relay_finalized_block_subscription_substrate_client = SubstrateClient::new(
                CONFIG.substrate.rpc_url.as_str(),
                CONFIG.substrate.network_id,
                CONFIG.substrate.connection_timeout_seconds,
                CONFIG.substrate.request_timeout_seconds,
            )
            .await?;
            let relay_substrate_client = Arc::new(Mutex::new(
                SubstrateClient::new(
                    CONFIG.substrate.rpc_url.as_str(),
                    CONFIG.substrate.network_id,
                    CONFIG.substrate.connection_timeout_seconds,
                    CONFIG.substrate.request_timeout_seconds,
                )
                .await?,
            ));
            let asset_hub_substrate_client = Arc::new(Mutex::new(
                SubstrateClient::new(
                    CONFIG.substrate.asset_hub_rpc_url.as_str(),
                    CONFIG.substrate.network_id,
                    CONFIG.substrate.connection_timeout_seconds,
                    CONFIG.substrate.request_timeout_seconds,
                )
                .await?,
            ));
            let relay_runtime_information = Arc::new(RwLock::new(RuntimeInformation::default()));
            let postgres = Arc::new(
                PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?,
            );
            // init extrinsic and event process error count metrics
            metrics::extrinsic_process_error_count()
                .set(postgres.get_extrinsic_process_error_log_count().await? as i64);
            metrics::event_process_error_count()
                .set(postgres.get_event_process_error_log_count().await? as i64);

            relay_finalized_block_subscription_substrate_client.subscribe_to_finalized_blocks(
                CONFIG.substrate.request_timeout_seconds,
                |finalized_block_header| async {
                    let error_cell = relay_error_cell.clone();
                    if let Some(error) = error_cell.get() {
                        return Err(anyhow::anyhow!("RELAY  {:?}", error));
                    }
                    let finalized_block_number = match finalized_block_header.get_number() {
                        Ok(block_number) => block_number,
                        Err(error) => {
                            log::error!("RELAY Cannot get block number for header: {finalized_block_header:?}");
                            return Err(anyhow::anyhow!("{error:?}"));
                        }
                    };
                    metrics::target_finalized_block_number().set(finalized_block_number as i64);
                    if RELAY_IS_BUSY.load(Ordering::SeqCst) {
                        log::debug!("RELAY Busy processing past blocks. Skip block #{finalized_block_number} for now.");
                        return Ok(());
                    }
                    let block_processor_substrate_client = relay_substrate_client.clone();
                    let asset_hub_substrate_client = asset_hub_substrate_client.clone();
                    let runtime_information = relay_runtime_information.clone();
                    let postgres = postgres.clone();
                    RELAY_IS_BUSY.store(true, Ordering::SeqCst);
                    tokio::spawn(async move {
                        let mut block_processor_substrate_client = block_processor_substrate_client.lock().await;
                        let mut asset_hub_substrate_client = asset_hub_substrate_client.lock().await;
                        let processed_block_height = match postgres.get_processed_block_height("relay").await {
                            Ok(processed_block_height) => processed_block_height,
                            Err(error) => {
                                log::error!("RELAY Cannot get processed block height from the database: {error:?}");
                                let _ = error_cell.set(error);
                                RELAY_IS_BUSY.store(false, Ordering::SeqCst);
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
                                    "RELAY Process block #{block_number}. Target #{finalized_block_number}.",
                                );
                                let start = std::time::Instant::now();
                                let process_result = self.process_relay_block(
                                    &mut block_processor_substrate_client,
                                    &mut asset_hub_substrate_client,
                                    &runtime_information,
                                    &postgres,
                                    block_number,
                                ).await;
                                metrics::block_processing_time_ms().observe(start.elapsed().as_millis() as f64);
                                match process_result {
                                    Ok(_) => {
                                        metrics::processed_finalized_block_number().set(block_number as i64);
                                        block_number += 1
                                    },
                                    Err(error) => {
                                        log::error!("RELAY {error:?}");
                                        log::error!(
                                            "RELAY History block processing failed for block #{block_number}.",
                                        );
                                        let _ = error_cell.set(error);
                                        break;
                                    }
                                }
                            }
                        } else {
                            log::info!("RELAY Process block #{finalized_block_number}.");
                            let start = std::time::Instant::now();
                            let update_result = self.process_relay_block(
                                &mut block_processor_substrate_client,
                                &mut asset_hub_substrate_client,
                                &runtime_information,
                                &postgres,
                                finalized_block_number,
                            ).await;
                            metrics::block_processing_time_ms().observe(start.elapsed().as_millis() as f64);
                            match update_result {
                                Ok(_) => {
                                    metrics::processed_finalized_block_number().set(finalized_block_number as i64);
                                },
                                Err(error) => {
                                    log::error!("RELAY {error:?}");
                                    log::error!(
                                        "RELAY Block processing failed for finalized block #{}. Will try again with the next block.",
                                        finalized_block_header.get_number().unwrap_or(0),
                                    );
                                    let _ = error_cell.set(error);
                                }
                            }
                        }
                        RELAY_IS_BUSY.store(false, Ordering::SeqCst);
                    });
                    Ok(())
                }).await;
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            log::error!(
                "RELAY Finalized block subscription exited. Will refresh connection and subscription after {delay_seconds} seconds.",
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
        }
    }

    async fn subscribe_asset_hub(&'static self) -> anyhow::Result<()> {
        loop {
            if ASSET_HUB_IS_BUSY.load(Ordering::SeqCst) {
                let delay_seconds = CONFIG.common.recovery_retry_seconds;
                log::warn!("ASSET_JHUB Busy processing past blocks. Hold re-instantiation for {delay_seconds} seconds.");
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
                continue;
            }
            let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
            let finalized_block_subscription_substrate_client = SubstrateClient::new(
                CONFIG.substrate.asset_hub_rpc_url.as_str(),
                CONFIG.substrate.network_id,
                CONFIG.substrate.connection_timeout_seconds,
                CONFIG.substrate.request_timeout_seconds,
            )
            .await?;
            let substrate_client = Arc::new(Mutex::new(
                SubstrateClient::new(
                    CONFIG.substrate.asset_hub_rpc_url.as_str(),
                    CONFIG.substrate.network_id,
                    CONFIG.substrate.connection_timeout_seconds,
                    CONFIG.substrate.request_timeout_seconds,
                )
                .await?,
            ));
            let relay_substrate_client = Arc::new(Mutex::new(
                SubstrateClient::new(
                    CONFIG.substrate.rpc_url.as_str(),
                    CONFIG.substrate.network_id,
                    CONFIG.substrate.connection_timeout_seconds,
                    CONFIG.substrate.request_timeout_seconds,
                )
                .await?,
            ));
            let postgres = Arc::new(
                PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?,
            );
            // init extrinsic and event process error count metrics
            metrics::extrinsic_process_error_count()
                .set(postgres.get_extrinsic_process_error_log_count().await? as i64);
            metrics::event_process_error_count()
                .set(postgres.get_event_process_error_log_count().await? as i64);

            finalized_block_subscription_substrate_client.subscribe_to_finalized_blocks(
                CONFIG.substrate.request_timeout_seconds,
                |finalized_block_header| async {
                    let error_cell = error_cell.clone();
                    if let Some(error) = error_cell.get() {
                        return Err(anyhow::anyhow!("ASSET_HUB  {:?}", error));
                    }
                    let finalized_block_number = match finalized_block_header.get_number() {
                        Ok(block_number) => block_number,
                        Err(error) => {
                            log::error!("ASSET_HUB Cannot get block number for header: {finalized_block_header:?}");
                            return Err(anyhow::anyhow!("{error:?}"));
                        }
                    };
                    metrics::target_finalized_block_number().set(finalized_block_number as i64);
                    if ASSET_HUB_IS_BUSY.load(Ordering::SeqCst) {
                        log::debug!("ASSET_HUB Busy processing past blocks. Skip block #{finalized_block_number} for now.");
                        return Ok(());
                    }
                    let block_processor_substrate_client = substrate_client.clone();
                    let relay_substrate_client = relay_substrate_client.clone();
                    let postgres = postgres.clone();
                    ASSET_HUB_IS_BUSY.store(true, Ordering::SeqCst);
                    tokio::spawn(async move {
                        let mut block_processor_substrate_client = block_processor_substrate_client.lock().await;
                        let mut relay_substrate_client = relay_substrate_client.lock().await;
                        let processed_block_height = match postgres.get_processed_block_height("asset_hub").await {
                            Ok(processed_block_height) => processed_block_height,
                            Err(error) => {
                                log::error!("ASSET_HUB Cannot get processed block height from the database: {error:?}");
                                let _ = error_cell.set(error);
                                RELAY_IS_BUSY.store(false, Ordering::SeqCst);
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
                                    "ASS_ETHUB Process block #{block_number}. Target #{finalized_block_number}.",
                                );
                                let start = std::time::Instant::now();
                                let process_result = self.process_asset_hub_block(
                                    &mut block_processor_substrate_client,
                                    &mut relay_substrate_client,
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
                                        log::error!("ASSET_HUB {error:?}");
                                        log::error!(
                                            "ASSET_HUB History block processing failed for block #{block_number}.",
                                        );
                                        let _ = error_cell.set(error);
                                        break;
                                    }
                                }
                            }
                        } else {
                            // update current era reward points every 3 minutes
                            let blocks_per_3_minutes = 3 * 60 * 1000
                                / get_metadata_expected_block_time_millis(&block_processor_substrate_client.metadata).unwrap();
                            log::info!("ASSET_HUB Process block #{finalized_block_number}.");
                            let start = std::time::Instant::now();
                            let update_result = self.process_asset_hub_block(
                                &mut block_processor_substrate_client,
                                &mut relay_substrate_client,
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
                                    log::error!("RELAY {error:?}");
                                    log::error!(
                                        "ASSET_HUB Block processing failed for finalized block #{}. Will try again with the next block.",
                                        finalized_block_header.get_number().unwrap_or(0),
                                    );
                                    let _ = error_cell.set(error);
                                }
                            }
                        }
                        ASSET_HUB_IS_BUSY.store(false, Ordering::SeqCst);
                    });
                    Ok(())
                }).await;
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            log::error!(
                "ASSET_HUB Finalized block subscription exited. Will refresh connection and subscription after {delay_seconds} seconds.",
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
        }
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
        self.subscribe_relay_chain().await?;
        self.subscribe_asset_hub().await?;
        Ok(())
    }
}
