//! Indexes historical block data into the PostreSQL database instance.

use async_lock::Mutex;
use async_recursion::async_recursion;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug, error, trace};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::metadata::MetadataVersion;
use subvt_types::{
    crypto::AccountId,
    substrate::{
        event::{ImOnlineEvent, StakingEvent, SubstrateEvent, SystemEvent, UtilityEvent},
        extrinsic::{
            ImOnlineExtrinsic, MultisigExtrinsic, ProxyExtrinsic, StakingExtrinsic,
            SubstrateExtrinsic, TimestampExtrinsic, UtilityExtrinsic,
        },
        Era, EraStakers, MultiAddress, ValidatorStake,
    },
};

lazy_static! {
    static ref CONFIG: Config = Config::default();
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
        debug!("Persist era #{} validators.", era.index);
        let all_validator_account_ids = substrate_client
            .get_all_validator_account_ids(block_hash)
            .await?;
        let bonded_account_id_map = substrate_client
            .get_bonded_account_id_map(&all_validator_account_ids, block_hash)
            .await?;
        let validator_stake_map = {
            let mut validator_stake_map: HashMap<AccountId, ValidatorStake> = HashMap::new();
            for validator_stake in &era_stakers.stakers {
                validator_stake_map
                    .insert(validator_stake.account.id.clone(), validator_stake.clone());
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
        debug!("Persisted era #{} validators and stakers.", era.index);
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
            debug!(
                "Era {} does not exist. Cannot persist reward points.",
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
        let mut era_reward_points_map: HashMap<AccountId, u32> = HashMap::new();
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
        debug!("Era {} rewards persisted.", era_index);
        Ok(())
    }

    async fn process_event(
        &self,
        substrate_client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        block_hash_epoch_index: (&str, u64),
        successful_extrinsic_indices: &mut Vec<u32>,
        failed_extrinsic_indices: &mut Vec<u32>,
        (event_index, event): (usize, &SubstrateEvent),
    ) -> anyhow::Result<()> {
        let (block_hash, epoch_index) = block_hash_epoch_index;
        match event {
            SubstrateEvent::ImOnline(im_online_event) => match im_online_event {
                ImOnlineEvent::HeartbeatReceived {
                    extrinsic_index,
                    im_online_key_hex_string,
                } => {
                    match substrate_client
                        .get_im_online_key_owner_account_id(block_hash, im_online_key_hex_string)
                        .await
                    {
                        Ok(validator_account_id) => {
                            let extrinsic_index =
                                extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                            postgres
                                .save_validator_heartbeart_event(
                                    block_hash,
                                    extrinsic_index,
                                    event_index as i32,
                                    epoch_index as i64,
                                    im_online_key_hex_string,
                                    &validator_account_id,
                                )
                                .await?;
                        }
                        Err(error) => {
                            error!("Cannot persist heartbeat event: {:?}", error);
                        }
                    }
                }
                ImOnlineEvent::SomeOffline {
                    identification_tuples,
                } => {
                    postgres
                        .save_validators_offline_event(
                            block_hash,
                            event_index as i32,
                            identification_tuples,
                        )
                        .await?;
                }
                _ => (),
            },
            SubstrateEvent::Staking(staking_event) => match staking_event {
                StakingEvent::Chilled {
                    extrinsic_index,
                    stash_account_id,
                } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_chilled_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                            stash_account_id,
                        )
                        .await?;
                }
                StakingEvent::EraPaid {
                    extrinsic_index,
                    era_index,
                    validator_payout,
                    remainder,
                } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_era_paid_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                            *era_index,
                            *validator_payout,
                            *remainder,
                        )
                        .await?;
                }
                StakingEvent::NominatorKicked {
                    extrinsic_index,
                    nominator_account_id,
                    validator_account_id,
                } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_nominator_kicked_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                            validator_account_id,
                            nominator_account_id,
                        )
                        .await?;
                }
                StakingEvent::Rewarded {
                    extrinsic_index,
                    rewardee_account_id,
                    amount,
                } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_rewarded_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                            rewardee_account_id,
                            *amount,
                        )
                        .await?;
                }
                StakingEvent::Slashed {
                    extrinsic_index,
                    validator_account_id,
                    amount,
                } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_slashed_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                            validator_account_id,
                            *amount,
                        )
                        .await?;
                }
                _ => (),
            },
            SubstrateEvent::System(system_event) => match system_event {
                SystemEvent::ExtrinsicFailed {
                    extrinsic_index,
                    dispatch_error: _,
                    dispatch_info: _,
                } => failed_extrinsic_indices.push(extrinsic_index.unwrap()),
                SystemEvent::ExtrinsicSuccess {
                    extrinsic_index,
                    dispatch_info: _,
                } => successful_extrinsic_indices.push(extrinsic_index.unwrap()),
                SystemEvent::NewAccount {
                    extrinsic_index,
                    account_id,
                } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_new_account_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                            account_id,
                        )
                        .await?;
                }
                SystemEvent::KilledAccount {
                    extrinsic_index,
                    account_id,
                } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_killed_account_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                            account_id,
                        )
                        .await?;
                }
                _ => (),
            },
            SubstrateEvent::Utility(utility_event) => match utility_event {
                UtilityEvent::ItemCompleted { extrinsic_index } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_batch_item_completed_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                        )
                        .await?;
                }
                UtilityEvent::BatchInterrupted {
                    extrinsic_index,
                    item_index,
                    dispatch_error,
                } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_batch_interrupted_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                            *item_index as i32,
                            format!("{:?}", dispatch_error),
                        )
                        .await?;
                }
                UtilityEvent::BatchCompleted { extrinsic_index } => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_batch_completed_event(block_hash, extrinsic_index, event_index as i32)
                        .await?;
                }
            },
            _ => (),
        }
        Ok(())
    }

    #[async_recursion]
    async fn process_extrinsic(
        &self,
        substrate_client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        (block_hash, block_number): (String, u64),
        active_validator_account_ids: &[AccountId],
        (index, is_nested_call, maybe_multisig_account_id, maybe_real_account_id, is_successful): (
            usize,
            bool,
            Option<AccountId>,
            Option<AccountId>,
            bool,
        ),
        extrinsic: &SubstrateExtrinsic,
    ) -> anyhow::Result<()> {
        match extrinsic {
            SubstrateExtrinsic::ImOnline(imonline_extrinsic) => match imonline_extrinsic {
                ImOnlineExtrinsic::Hearbeat {
                    maybe_signature: _,
                    block_number,
                    session_index,
                    validator_index,
                } => {
                    if let Some(validator_account_id) =
                        active_validator_account_ids.get(*validator_index as usize)
                    {
                        let _ = postgres
                            .save_heartbeat_extrinsic(
                                &block_hash,
                                index as i32,
                                is_nested_call,
                                is_successful,
                                (*block_number, *session_index),
                                (*validator_index, validator_account_id),
                            )
                            .await?;
                    } else {
                        error!(
                            "Cannot find active validator account id with index {}. Cannot persist heartbeat extrinsic in block {}.",
                            validator_index,
                            block_hash
                        );
                    }
                }
            },
            SubstrateExtrinsic::Multisig(multisig_extrinsic) => match multisig_extrinsic {
                MultisigExtrinsic::AsMulti {
                    maybe_signature: signature,
                    threshold,
                    other_signatories,
                    maybe_timepoint: _,
                    call,
                    store_call: _,
                    max_weight: _,
                } => {
                    let signature = if let Some(signature) = signature {
                        signature
                    } else {
                        error!(
                            "Cannot get signature while processing AsMulti extrinsic {}-{}.",
                            block_number, index
                        );
                        return Ok(());
                    };
                    let multisig_account_id = if let Some(signer_account_id) =
                        signature.get_signer_account_id()
                    {
                        AccountId::multisig_account_id(
                            &signer_account_id,
                            other_signatories,
                            *threshold,
                        )
                    } else {
                        error!("Cannot get multisig account id while processing AsMulti extrinsic {}-{}.", block_number, index);
                        return Ok(());
                    };
                    self.process_extrinsic(
                        substrate_client,
                        postgres,
                        (block_hash, block_number),
                        active_validator_account_ids,
                        (index, true, Some(multisig_account_id), None, is_successful),
                        call,
                    )
                    .await?;
                }
                MultisigExtrinsic::AsMultiThreshold1 {
                    maybe_signature: signature,
                    other_signatories,
                    call,
                } => {
                    let signature = if let Some(signature) = signature {
                        signature
                    } else {
                        error!("Cannot get signature while processing AsMultiThreshold1 extrinsic {}-{}.", block_number, index);
                        return Ok(());
                    };
                    let multisig_account_id = if let Some(signer_account_id) =
                        signature.get_signer_account_id()
                    {
                        AccountId::multisig_account_id(&signer_account_id, other_signatories, 1)
                    } else {
                        error!("Cannot get multisig account id while processing AsMultiThreshold1 extrinsic {}-{}.", block_number, index);
                        return Ok(());
                    };
                    self.process_extrinsic(
                        substrate_client,
                        postgres,
                        (block_hash, block_number),
                        active_validator_account_ids,
                        (index, true, Some(multisig_account_id), None, is_successful),
                        call,
                    )
                    .await?;
                }
            },
            SubstrateExtrinsic::Proxy(proxy_extrinsic) => match proxy_extrinsic {
                ProxyExtrinsic::Proxy {
                    maybe_signature: _,
                    real_account_id,
                    force_proxy_type: _,
                    call,
                } => {
                    self.process_extrinsic(
                        substrate_client,
                        postgres,
                        (block_hash, block_number),
                        active_validator_account_ids,
                        (
                            index,
                            true,
                            maybe_multisig_account_id,
                            Some(real_account_id.clone()),
                            is_successful,
                        ),
                        call,
                    )
                    .await?;
                }
                ProxyExtrinsic::ProxyAnnounced {
                    maybe_signature: _,
                    delegate_account_id: _,
                    real_account_id,
                    force_proxy_type: _,
                    call,
                } => {
                    self.process_extrinsic(
                        substrate_client,
                        postgres,
                        (block_hash, block_number),
                        active_validator_account_ids,
                        (
                            index,
                            true,
                            maybe_multisig_account_id,
                            Some(real_account_id.clone()),
                            is_successful,
                        ),
                        call,
                    )
                    .await?;
                }
            },
            SubstrateExtrinsic::Staking(staking_extrinsic) => match staking_extrinsic {
                StakingExtrinsic::Bond {
                    maybe_signature: signature,
                    controller,
                    amount,
                    reward_destination,
                } => {
                    let maybe_stash_account_id =
                        if let Some(real_account_id) = maybe_real_account_id {
                            Some(real_account_id)
                        } else if let Some(multisig_account_id) = maybe_multisig_account_id {
                            Some(multisig_account_id)
                        } else {
                            match signature {
                                Some(signature) => signature.get_signer_account_id(),
                                _ => None,
                            }
                        };
                    let controller_account_id = if let Some(account_id) =
                        controller.get_account_id()
                    {
                        account_id
                    } else {
                        error!("Controller address is not raw account id in staking.bond. Cannot persist.");
                        return Ok(());
                    };
                    if let Some(stash_account_id) = maybe_stash_account_id {
                        postgres
                            .save_bond_extrinsic(
                                &block_hash,
                                index as i32,
                                is_nested_call,
                                is_successful,
                                (
                                    &stash_account_id,
                                    &controller_account_id,
                                    *amount,
                                    reward_destination,
                                ),
                            )
                            .await?;
                    } else {
                        error!("Cannot get caller account id from signature for extrinsic #{} Staking.bond.", index);
                    }
                }
                StakingExtrinsic::Nominate {
                    maybe_signature: signature,
                    targets,
                } => {
                    let maybe_controller_account_id =
                        if let Some(real_account_id) = maybe_real_account_id {
                            Some(real_account_id)
                        } else if let Some(multisig_account_id) = maybe_multisig_account_id {
                            Some(multisig_account_id)
                        } else {
                            match signature {
                                Some(signature) => signature.get_signer_account_id(),
                                _ => None,
                            }
                        };
                    if let Some(controller_account_id) = maybe_controller_account_id {
                        let target_account_ids: Vec<AccountId> = targets
                            .iter()
                            .filter_map(|target_multi_address| match target_multi_address {
                                MultiAddress::Id(account_id) => Some(account_id.clone()),
                                _ => {
                                    error!("Unsupported multi-address type for nomination target.");
                                    None
                                }
                            })
                            .collect();
                        postgres
                            .save_nomination(
                                &block_hash,
                                index as i32,
                                is_nested_call,
                                is_successful,
                                &controller_account_id,
                                &target_account_ids,
                            )
                            .await?;
                    } else {
                        error!("Cannot get nominator account id from signature for extrinsic #{} Staking.nominate.", index);
                    }
                }
                StakingExtrinsic::PayoutStakers {
                    maybe_signature: signature,
                    validator_account_id,
                    era_index,
                } => {
                    let maybe_caller_account_id =
                        if let Some(multisig_account_id) = maybe_multisig_account_id {
                            Some(multisig_account_id)
                        } else if let Some(real_account_id) = maybe_real_account_id {
                            Some(real_account_id)
                        } else {
                            match signature {
                                Some(signature) => signature.get_signer_account_id(),
                                _ => None,
                            }
                        };
                    if let Some(caller_account_id) = maybe_caller_account_id {
                        // ignore the errors here - may fail due to non-existent era foreign key,
                        // past eras may not have been saved
                        let _ = postgres
                            .save_payout_stakers_extrinsic(
                                &block_hash,
                                index as i32,
                                is_nested_call,
                                is_successful,
                                (&caller_account_id, validator_account_id),
                                *era_index,
                            )
                            .await;
                    } else {
                        error!("Cannot get caller account id from signature for extrinsic #{} Staking.payout_stakers.", index);
                    }
                }
                StakingExtrinsic::SetController {
                    maybe_signature: signature,
                    controller,
                } => {
                    let maybe_caller_account_id =
                        if let Some(multisig_account_id) = maybe_multisig_account_id {
                            Some(multisig_account_id)
                        } else if let Some(real_account_id) = maybe_real_account_id {
                            Some(real_account_id)
                        } else {
                            match signature {
                                Some(signature) => signature.get_signer_account_id(),
                                _ => None,
                            }
                        };
                    let controller_account_id = if let Some(account_id) =
                        controller.get_account_id()
                    {
                        account_id
                    } else {
                        error!("Controller address is not raw account id in staking.set_controller. Cannot persist.");
                        return Ok(());
                    };
                    if let Some(caller_account_id) = maybe_caller_account_id {
                        postgres
                            .save_set_controller_extrinsic(
                                &block_hash,
                                index as i32,
                                is_nested_call,
                                is_successful,
                                &caller_account_id,
                                &controller_account_id,
                            )
                            .await?;
                    } else {
                        error!("Cannot get caller account id from signature for extrinsic #{} Staking.payout_stakers.", index);
                    }
                }
                StakingExtrinsic::Validate {
                    maybe_signature: signature,
                    preferences,
                } => {
                    let maybe_controller_account_id =
                        if let Some(multisig_account_id) = maybe_multisig_account_id {
                            Some(multisig_account_id)
                        } else if let Some(real_account_id) = maybe_real_account_id {
                            Some(real_account_id)
                        } else {
                            match signature {
                                Some(signature) => signature.get_signer_account_id(),
                                _ => None,
                            }
                        };
                    if let Some(controller_account_id) = maybe_controller_account_id {
                        // get stash account id
                        if let Some(stash_account_id) = substrate_client
                            .get_stash_account_id(&controller_account_id, &block_hash)
                            .await?
                        {
                            postgres
                                .save_validate_extrinsic(
                                    &block_hash,
                                    index as i32,
                                    is_nested_call,
                                    is_successful,
                                    (&stash_account_id, &controller_account_id),
                                    preferences,
                                )
                                .await?;
                        } else {
                            error!(
                                "Cannot get stash account id for controller {}.",
                                controller_account_id.to_string()
                            );
                        }
                    } else {
                        error!("Cannot get controller account id from signature for extrinsic #{} Staking.validate.", index);
                    }
                }
            },
            SubstrateExtrinsic::Utility(utility_extrinsic) => match utility_extrinsic {
                UtilityExtrinsic::Batch {
                    maybe_signature: _,
                    calls,
                } => {
                    for call in calls {
                        self.process_extrinsic(
                            substrate_client,
                            postgres,
                            (block_hash.clone(), block_number),
                            active_validator_account_ids,
                            (
                                index,
                                true,
                                maybe_multisig_account_id.clone(),
                                None,
                                is_successful,
                            ),
                            call,
                        )
                        .await?;
                    }
                }
                UtilityExtrinsic::BatchAll {
                    maybe_signature: _,
                    calls,
                } => {
                    for call in calls {
                        self.process_extrinsic(
                            substrate_client,
                            postgres,
                            (block_hash.clone(), block_number),
                            active_validator_account_ids,
                            (
                                index,
                                true,
                                maybe_multisig_account_id.clone(),
                                None,
                                is_successful,
                            ),
                            call,
                        )
                        .await?;
                    }
                }
            },
            _ => (),
        }
        Ok(())
    }

    async fn process_block(
        &self,
        substrate_client: &mut SubstrateClient,
        runtime_information: &Arc<RwLock<RuntimeInformation>>,
        postgres: &PostgreSQLNetworkStorage,
        block_number: u64,
    ) -> anyhow::Result<()> {
        debug!("Process block #{}.", block_number);
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
            debug!(
                "Different runtime version #{} than client's #{}. Will reset metadata.",
                runtime_upgrade_info.spec_version,
                substrate_client
                    .metadata
                    .last_runtime_upgrade_info
                    .spec_version
            );
            substrate_client.set_metadata_at_block(&block_hash).await?;
            debug!(
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
        let current_epoch_index = substrate_client
            .get_current_epoch_index(&block_hash)
            .await?;
        let active_validator_account_ids = substrate_client
            .get_active_validator_account_ids(&block_hash)
            .await?;

        if last_epoch_index != current_epoch_index || last_era_index != active_era.index {
            let era_stakers = substrate_client
                .get_era_stakers(&active_era, true, &block_hash)
                .await?;
            if last_epoch_index != current_epoch_index {
                debug!("New epoch. Persist era if it doesn't exist.");
                let total_stake = substrate_client
                    .get_era_total_stake(active_era.index, &block_hash)
                    .await?;
                postgres
                    .save_era(&active_era, total_stake, &era_stakers)
                    .await?;
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
        // update current era reward points every 10 minutes
        let blocks_per_10_minutes = 10 * 60 * 1000
            / substrate_client
                .metadata
                .constants
                .expected_block_time_millis;
        if block_number % blocks_per_10_minutes == 0 {
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
            runtime_information.epoch_index = current_epoch_index;
        }
        let events = substrate_client.get_block_events(&block_hash).await?;
        debug!("Got #{} events for block #{}.", events.len(), block_number);
        let extrinsics = substrate_client.get_block_extrinsics(&block_hash).await?;
        debug!(
            "Got #{} extrinsics for block #{}.",
            extrinsics.len(),
            block_number
        );

        let mut block_timestamp: Option<u64> = None;
        for extrinsic in &extrinsics {
            if let SubstrateExtrinsic::Timestamp(timestamp_extrinsic) = extrinsic {
                match timestamp_extrinsic {
                    TimestampExtrinsic::Set {
                        maybe_signature: _,
                        timestamp,
                    } => {
                        block_timestamp = Some(*timestamp);
                    }
                }
            }
        }
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
                (active_era.index, current_epoch_index as u32),
                (metadata_version, runtime_version),
            )
            .await?;
        // process/persist events
        let mut successful_extrinsic_indices: Vec<u32> = Vec::new();
        let mut failed_extrinsic_indices: Vec<u32> = Vec::new();
        for (index, event) in events.iter().enumerate() {
            self.process_event(
                substrate_client,
                postgres,
                (&block_hash, current_epoch_index),
                &mut successful_extrinsic_indices,
                &mut failed_extrinsic_indices,
                (index, event),
            )
            .await?;
        }
        // persist extrinsics
        for (index, extrinsic) in extrinsics.iter().enumerate() {
            // check events for batch & batch_all
            let is_successful = successful_extrinsic_indices.contains(&(index as u32));
            self.process_extrinsic(
                substrate_client,
                postgres,
                (block_hash.clone(), block_number),
                &active_validator_account_ids,
                (index, false, None, None, is_successful),
                extrinsic,
            )
            .await?
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
    async fn run(&'static self) -> anyhow::Result<()> {
        loop {
            let block_subscription_substrate_client = SubstrateClient::new(&CONFIG).await?;
            let block_processor_substrate_client =
                Arc::new(Mutex::new(SubstrateClient::new(&CONFIG).await?));
            let runtime_information = Arc::new(RwLock::new(RuntimeInformation::default()));
            let postgres = Arc::new(
                PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?,
            );
            let is_indexing_past_blocks = Arc::new(AtomicBool::new(false));

            block_subscription_substrate_client.subscribe_to_finalized_blocks(|finalized_block_header| {
                let finalized_block_number = match finalized_block_header.get_number() {
                    Ok(block_number) => block_number,
                    Err(_) => return error!("Cannot get block number for header: {:?}", finalized_block_header)
                };
                let block_processor_substrate_client = block_processor_substrate_client.clone();
                let runtime_information = runtime_information.clone();
                let postgres = postgres.clone();
                if is_indexing_past_blocks.load(Ordering::SeqCst) {
                    trace!("Busy indexing past blocks. Skip block #{} for now.", finalized_block_number);
                    return;
                }
                let is_indexing_past_blocks = Arc::clone(&is_indexing_past_blocks);

                tokio::spawn(async move {
                    let mut block_processor_substrate_client = block_processor_substrate_client.lock().await;
                    let processed_block_height = match postgres.get_processed_block_height().await {
                        Ok(processed_block_height) => processed_block_height,
                        Err(error) => {
                            error!("Cannot get processed block height from the database: {:?}", error);
                            return;
                        }
                    };
                    if ((processed_block_height + 1) as u64) < finalized_block_number {
                        is_indexing_past_blocks.store(true, Ordering::SeqCst);
                        let mut block_number = std::cmp::max(
                            (processed_block_height + 1) as u64,
                            CONFIG.block_processor.start_block_number
                        );
                        while block_number <= finalized_block_number {
                            let update_result = self.process_block(
                                &mut block_processor_substrate_client,
                                &runtime_information,
                                &postgres,
                                block_number,
                            ).await;
                            match update_result {
                                Ok(_) => block_number += 1,
                                Err(error) => {
                                    error!("{:?}", error);
                                    error!(
                                        "History block processing failed for block #{}.",
                                        block_number,
                                    );
                                    is_indexing_past_blocks.store(false, Ordering::SeqCst);
                                    return;
                                }
                            }
                        }
                        is_indexing_past_blocks.store(false, Ordering::SeqCst);
                    } else {
                        let update_result = self.process_block(
                            &mut block_processor_substrate_client,
                            &runtime_information,
                            &postgres,
                            finalized_block_number,
                        ).await;
                        match update_result {
                            Ok(_) => (),
                            Err(error) => {
                                error!("{:?}", error);
                                error!(
                                "Block processing failed for finalized block #{}. Will try again with the next block.",
                                finalized_block_header.get_number().unwrap_or(0),
                            );
                            }
                        }
                    }
                });
            }).await?;
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            error!(
                "Finalized block subscription exited. Will refresh connection and subscription after {} seconds.",
                delay_seconds
            );
            std::thread::sleep(std::time::Duration::from_secs(delay_seconds));
        }
    }
}
