//! SubVT Substrate client implementation.
#![warn(clippy::disallowed_types)]

use crate::storage_utility::{
    get_rpc_paged_keys_params, get_rpc_paged_map_keys_params, get_rpc_storage_map_params,
    get_rpc_storage_plain_params, get_storage_double_map_key, get_storage_map_key,
};
use async_recursion::async_recursion;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed, RuntimeMetadataV14};
use jsonrpsee::ws_client::WsClient;
use jsonrpsee::{
    core::client::{Client, ClientT, Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
use parity_scale_codec::Decode;
use rustc_hash::{FxHashMap as HashMap, FxHasher};
use sp_core::storage::{StorageChangeSet, StorageKey};
use sp_core::ConstU32;
use std::cmp::max;
use std::collections::BTreeMap;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use subvt_types::app::event::democracy::{AccountVote, ConvictionVote};
use subvt_types::crypto::AccountId;
use subvt_types::substrate::democracy::{
    get_democracy_conviction_u8, DelegatedVote, DirectVote, ReferendumVote, VoteType,
};
use subvt_types::substrate::error::DecodeError;
use subvt_types::substrate::legacy::LegacyCoreOccupied;
use subvt_types::substrate::metadata::{
    get_metadata_epoch_duration_millis, get_metadata_era_duration_millis,
};
use subvt_types::substrate::para::ParaCoreAssignment;
use subvt_types::substrate::{
    event::SubstrateEvent, extrinsic::SubstrateExtrinsic, legacy::LegacyValidatorPrefs, Account,
    Balance, Block, BlockHeader, BlockNumber, BlockWrapper, Chain, ConvictionVoting,
    CoreAssignment, DemocracyVoting, Epoch, Era, EraRewardPoints, EraStakers, IdentityRegistration,
    LastRuntimeUpgradeInfo, Nomination, PagedExposureMetadata, RewardDestination,
    ScrapedOnChainVotes, Stake, SuperAccountId, SystemProperties, ValidatorPreferences,
    ValidatorStake,
};
/// Substrate client structure and its functions.
/// This is the main gateway for SubVT to a Substrate node RPC interface.
use subvt_types::subvt::ValidatorDetails;
use subvt_utility::decode_hex_string;
use tokio::time::timeout;

mod storage_utility;

const KEY_QUERY_PAGE_SIZE: usize = 500;

/// The client.
pub struct SubstrateClient {
    network_id: u32,
    pub chain: Chain,
    pub metadata: RuntimeMetadataV14,
    pub system_properties: SystemProperties,
    ws_client: Client,
    pub last_runtime_upgrade_info: LastRuntimeUpgradeInfo,
}

async fn get_metadata_at_block(
    ws_client: &WsClient,
    block_hash: &str,
) -> anyhow::Result<RuntimeMetadataV14> {
    let metadata_hex_string: String = ws_client
        .request("state_getMetadata", rpc_params!(block_hash))
        .await?;
    let metadata_hex_string = metadata_hex_string.trim_start_matches("0x");
    let mut metadata_hex_decoded: &[u8] = &hex::decode(metadata_hex_string)?;
    let metadata_prefixed = RuntimeMetadataPrefixed::decode(&mut metadata_hex_decoded)?;
    let metadata = match metadata_prefixed.1 {
        RuntimeMetadata::V14(metadata) => metadata,
        _ => panic!("Unsupported metadata version."),
    };
    Ok(metadata)
}

impl SubstrateClient {
    /// Connect to the node and construct a new Substrate client.
    pub async fn new(
        rpc_url: &str,
        network_id: u32,
        connection_timeout_seconds: u64,
        request_timeout_seconds: u64,
    ) -> anyhow::Result<Self> {
        log::info!("Constructing Substrate client.");
        let ws_client = WsClientBuilder::default()
            .connection_timeout(std::time::Duration::from_secs(connection_timeout_seconds))
            .request_timeout(std::time::Duration::from_secs(request_timeout_seconds))
            .build(rpc_url)
            .await?;
        log::info!("Substrate connection successful.");
        // get current block hash
        let block_hash: String = ws_client
            .request("chain_getBlockHash", rpc_params!())
            .await?;
        let chain: String = ws_client.request("system_chain", rpc_params!()).await?;
        let chain = Chain::from_str(chain.as_str())?;
        let metadata = get_metadata_at_block(&ws_client, &block_hash).await?;
        log::info!("Got metadata.");
        let last_runtime_upgrade_hex_string: String = ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("System", "LastRuntimeUpgrade", Some(&block_hash)),
            )
            .await?;
        let last_runtime_upgrade_info =
            LastRuntimeUpgradeInfo::from_substrate_hex_string(last_runtime_upgrade_hex_string)?;
        // subvt_types::substrate::metadata::print_metadata_type_codes(&metadata)?;
        log::info!("Got last runtime upgrade info.");
        let system_properties: SystemProperties = ws_client
            .request("system_properties", rpc_params!())
            .await?;
        log::info!("Got system properties. {system_properties:?}");
        Ok(Self {
            network_id,
            chain,
            metadata,
            system_properties,
            ws_client,
            last_runtime_upgrade_info,
        })
    }

    pub async fn set_metadata_at_block(
        &mut self,
        block_number: u64,
        block_hash: &str,
    ) -> anyhow::Result<()> {
        let prev_block_hash = self.get_block_hash(block_number - 1).await?;
        let metadata = get_metadata_at_block(&self.ws_client, &prev_block_hash).await?;
        self.last_runtime_upgrade_info = self.get_last_runtime_upgrade_info(block_hash).await?;
        self.metadata = metadata;
        Ok(())
    }

    pub async fn get_current_block_hash(&self) -> anyhow::Result<String> {
        let hash = self
            .ws_client
            .request("chain_getBlockHash", rpc_params!())
            .await?;
        Ok(hash)
    }

    /// Get a block hash by its number.
    pub async fn get_block_hash(&self, block_number: u64) -> anyhow::Result<String> {
        let hash: String = self
            .ws_client
            .request("chain_getBlockHash", rpc_params!(block_number))
            .await?;
        Ok(format!(
            "0x{}",
            hash.trim_start_matches("0x").to_uppercase()
        ))
    }

    /// Get a block header by its hash.
    pub async fn get_block_header(&self, block_hash: &str) -> anyhow::Result<BlockHeader> {
        let mut header: BlockHeader = self
            .ws_client
            .request("chain_getHeader", rpc_params!(&block_hash))
            .await?;
        header.parent_hash = format!(
            "0x{}",
            header.parent_hash.trim_start_matches("0x").to_uppercase()
        );
        header.extrinsics_root = format!(
            "0x{}",
            header
                .extrinsics_root
                .trim_start_matches("0x")
                .to_uppercase()
        );
        header.state_root = format!(
            "0x{}",
            header.state_root.trim_start_matches("0x").to_uppercase()
        );
        Ok(header)
    }

    /// Get the hash of the current finalized block.
    pub async fn get_finalized_block_hash(&self) -> anyhow::Result<String> {
        let hash: String = self
            .ws_client
            .request("chain_getFinalizedHead", rpc_params!())
            .await?;
        Ok(format!(
            "0x{}",
            hash.trim_start_matches("0x").to_uppercase()
        ))
    }

    /// Get a block.
    async fn get_block(&self, block_hash: &str) -> anyhow::Result<Block> {
        let mut block_wrapper: BlockWrapper = self
            .ws_client
            .request("chain_getBlock", rpc_params!(&block_hash))
            .await?;
        block_wrapper.block.header.parent_hash = format!(
            "0x{}",
            block_wrapper
                .block
                .header
                .parent_hash
                .trim_start_matches("0x")
                .to_uppercase()
        );
        Ok(block_wrapper.block)
    }

    pub async fn get_block_timestamp(&self, block_hash: &str) -> anyhow::Result<u64> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Timestamp", "Now", Some(block_hash)),
            )
            .await?;
        decode_hex_string(hex_string.as_str())
    }

    /// Get active era at the given block.
    pub async fn get_active_era(
        &self,
        block_hash: &str,
        babe_metadata: &RuntimeMetadataV14,
    ) -> anyhow::Result<Era> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Staking", "ActiveEra", Some(block_hash)),
            )
            .await?;
        let active_era_info = Era::from(
            hex_string.as_str(),
            get_metadata_era_duration_millis(babe_metadata, &self.metadata)?,
        )?;
        Ok(active_era_info)
    }

    /// Get the index of the epoch at the given block hash.
    pub async fn get_current_epoch_index(&self, block_hash: &str) -> anyhow::Result<u64> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Babe", "EpochIndex", Some(block_hash)),
            )
            .await?;
        let index = decode_hex_string(hex_string.as_str())?;
        Ok(index)
    }

    /// Get current epoch at the given block.
    pub async fn get_current_epoch(&self, era: &Era, block_hash: &str) -> anyhow::Result<Epoch> {
        let index = self.get_current_epoch_index(block_hash).await?;
        let start_block_number = {
            let hex_string: String = self
                .ws_client
                .request(
                    "state_getStorage",
                    get_rpc_storage_plain_params("Babe", "EpochStart", Some(block_hash)),
                )
                .await?;
            decode_hex_string::<(u32, u32)>(hex_string.as_str())?.1
        };
        let start_block_hash = self.get_block_hash(start_block_number as u64).await?;
        let start_timestamp = self.get_block_timestamp(&start_block_hash).await?;
        let end_timestamp = start_timestamp + get_metadata_epoch_duration_millis(&self.metadata)?;
        Ok(Epoch {
            index,
            era_index: era.index,
            start_block_number,
            start_timestamp,
            end_timestamp,
        })
    }

    /// Decode account if from a transparent key.
    fn account_id_from_storage_key_string(&self, storage_key_string: &str) -> AccountId {
        let hex_string = &storage_key_string[(storage_key_string.len() - 64)..];
        decode_hex_string(hex_string).unwrap()
    }

    fn account_id_from_storage_key(&self, storage_key: &StorageKey) -> AccountId {
        storage_key.0[storage_key.0.len() - 32..]
            .try_into()
            .unwrap()
    }

    /// Get controller account id for a given stash account id at the given block.
    pub async fn get_controller_account_id(
        &self,
        stash_account_id: &AccountId,
        maybe_block_hash: Option<&str>,
    ) -> anyhow::Result<Option<AccountId>> {
        let storage_key =
            get_storage_map_key(&self.metadata, "Staking", "Bonded", stash_account_id);
        let mut params = rpc_params!(vec![storage_key]);
        if let Some(block_hash) = maybe_block_hash {
            params.insert(block_hash)?;
        }
        let chunk_values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request("state_queryStorageAt", params)
            .await?;
        if let Some(value) = chunk_values.first() {
            if let Some((_, Some(data))) = value.changes.first() {
                let bytes: [u8; 32] = (&data.0 as &[u8]).try_into()?;
                return Ok(Some(AccountId::from(bytes)));
            }
        }
        Ok(None)
    }

    /// Get the ledger for a controller account at the given block.
    pub async fn get_stake(
        &self,
        controller_account_id: &AccountId,
        maybe_block_hash: Option<&str>,
    ) -> anyhow::Result<Option<Stake>> {
        let storage_key =
            get_storage_map_key(&self.metadata, "Staking", "Ledger", controller_account_id);
        let mut params = rpc_params!(vec![storage_key]);
        if let Some(block_hash) = maybe_block_hash {
            params.insert(block_hash)?;
        }
        let chunk_values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request("state_queryStorageAt", params)
            .await?;
        if let Some(value) = chunk_values.first() {
            if let Some((_, Some(data))) = value.changes.first() {
                let stake = Stake::from_bytes(&data.0 as &[u8])?;
                return Ok(Some(stake));
            }
        }
        Ok(None)
    }

    /// Get the stash account id for a controller account id at the given block.
    pub async fn get_stash_account_id(
        &self,
        controller_account_id: &AccountId,
        maybe_block_hash: Option<&str>,
    ) -> anyhow::Result<Option<AccountId>> {
        match self
            .get_stake(controller_account_id, maybe_block_hash)
            .await?
        {
            Some(stake) => Ok(Some(stake.stash_account_id)),
            None => Ok(None),
        }
    }

    /// Get the list of the account ids of all validators (active and inactive) at the given block.
    pub async fn get_all_validator_account_ids(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<AccountId>> {
        let all_validator_ids: Vec<AccountId> = self
            .get_all_keys_for_storage("Staking", "Validators", block_hash)
            .await?
            .into_iter()
            .map(|key| self.account_id_from_storage_key_string(&key))
            .collect();
        Ok(all_validator_ids)
    }

    /// Gets the map from stash account ids to controller account ids at the given block
    /// for the given stash account ids.
    pub async fn get_bonded_account_id_map(
        &self,
        account_ids: &[AccountId],
        block_hash: &str,
    ) -> anyhow::Result<HashMap<AccountId, AccountId>> {
        let mut map = HashMap::default();
        let keys: Vec<String> = account_ids
            .iter()
            .map(|account_id| get_storage_map_key(&self.metadata, "Staking", "Bonded", account_id))
            .collect();
        if keys.is_empty() {
            return Ok(HashMap::default());
        }
        for chunk in keys.chunks(KEY_QUERY_PAGE_SIZE) {
            let chunk_values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                .await?;
            for (storage_key, data) in &chunk_values[0].changes {
                if let Some(data) = data {
                    let bytes: [u8; 32] = (&data.0 as &[u8]).try_into()?;
                    map.insert(
                        self.account_id_from_storage_key(storage_key),
                        AccountId::from(bytes),
                    );
                }
            }
        }
        Ok(map)
    }

    /// Get the list of all active validators' stash account ids at the given block.
    pub async fn get_active_validator_account_ids(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<AccountId>> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Session", "Validators", Some(block_hash)),
            )
            .await?;
        let account_ids = decode_hex_string(hex_string.as_str())?;
        Ok(account_ids)
    }

    /// Maps the given accounts ids to tuples that contain the parent account id and child display.
    /// Returned map will not contain an entry for the account id that has no parent.
    pub async fn get_parent_account_ids(
        &self,
        account_ids: &[AccountId],
        block_hash: &str,
    ) -> anyhow::Result<HashMap<AccountId, (AccountId, Option<String>)>> {
        let identity_hash = match self.chain {
            Chain::Kusama => "e86e8022fc71349382f6c23cea028124eda34ab7acd7f07bee8374dbb33f7674",
            _ => block_hash,
        };
        let keys: Vec<String> = account_ids
            .iter()
            .map(|account_id| {
                get_storage_map_key(&self.metadata, "Identity", "SuperOf", &account_id)
            })
            .collect();
        log::trace!("Got {} keys for super accounts.", keys.len());
        if keys.is_empty() {
            return Ok(HashMap::default());
        }
        let values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request("state_queryStorageAt", rpc_params!(keys, &identity_hash))
            .await?;
        log::trace!(
            "Got {} optional super accounts records.",
            values[0].changes.len()
        );
        let mut parent_account_map: HashMap<AccountId, (AccountId, Option<String>)> =
            HashMap::default();
        for (storage_key, storage_data) in values[0].changes.iter() {
            if let Some(data) = storage_data {
                let account_id = self.account_id_from_storage_key(storage_key);
                let mut bytes: &[u8] = &data.0;
                let super_identity: SuperAccountId = Decode::decode(&mut bytes).unwrap();
                parent_account_map.insert(
                    account_id,
                    (
                        super_identity.0,
                        subvt_types::substrate::data_to_string(super_identity.1),
                    ),
                );
            }
        }
        log::trace!(
            "Got {} super accounts. Get identities for super accounts.",
            parent_account_map.len()
        );
        Ok(parent_account_map)
    }

    /// Get identity records for the given account ids at the given block.
    pub async fn get_identities(
        &self,
        account_ids: &[AccountId],
        block_hash: &str,
    ) -> anyhow::Result<HashMap<AccountId, IdentityRegistration>> {
        let keys: Vec<String> = account_ids
            .iter()
            .map(|account_id| {
                get_storage_map_key(&self.metadata, "Identity", "IdentityOf", account_id)
            })
            .collect();
        log::trace!("Got {} storage keys for identities.", keys.len());
        if keys.is_empty() {
            return Ok(HashMap::default());
        }
        let values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request("state_queryStorageAt", rpc_params!(keys, block_hash))
            .await?;
        log::trace!("Got {} optional identities.", values[0].changes.len());
        let mut identity_map: HashMap<AccountId, IdentityRegistration> = HashMap::default();
        for (storage_key, storage_data) in values[0].changes.iter() {
            let account_id = self.account_id_from_storage_key(storage_key);
            if let Some(data) = storage_data {
                let bytes: &[u8] = &data.0;
                identity_map.insert(account_id, IdentityRegistration::from_bytes(bytes).unwrap());
            }
        }
        Ok(identity_map)
    }

    /// Get complete account details for the given account ids at the given block.
    pub async fn get_accounts(
        &self,
        account_ids: &[AccountId],
        fetch_parent_accounts: bool,
        block_hash: &str,
    ) -> anyhow::Result<Vec<Account>> {
        let identity_map = { self.get_identities(account_ids, block_hash).await? };
        let parent_account_id_map = { self.get_parent_account_ids(account_ids, block_hash).await? };
        let parent_account_identity_map = {
            let super_account_ids: Vec<AccountId> =
                parent_account_id_map.values().map(|pair| pair.0).collect();
            self.get_identities(&super_account_ids, block_hash).await?
        };
        let accounts: Vec<Account> = account_ids
            .iter()
            .cloned()
            .map(|account_id| {
                let mut account = Account {
                    id: account_id,
                    address: account_id.to_ss58_check(),
                    ..Default::default()
                };
                if let Some(identity) = identity_map.get(&account_id) {
                    account.identity = Some(identity.clone());
                }
                if let Some(parent_account_id) = parent_account_id_map.get(&account_id) {
                    if fetch_parent_accounts {
                        let mut parent_account = Account {
                            id: parent_account_id.0,
                            address: parent_account_id.0.to_ss58_check(),
                            ..Default::default()
                        };
                        if let Some(parent_account_identity) =
                            parent_account_identity_map.get(&parent_account_id.0)
                        {
                            parent_account.identity = Some(parent_account_identity.clone());
                        }
                        account.parent = Box::new(Some(parent_account));
                        account.child_display.clone_from(&parent_account_id.1);
                    } else {
                        account.parent_account_id = Some(parent_account_id.0);
                        account.child_display.clone_from(&parent_account_id.1);
                    }
                }
                account
            })
            .collect();
        Ok(accounts)
    }

    /// Get the complete keys for the given module (pallet) and storage.
    /// An example would be the complete keys for `Staking.Nominators`.
    async fn get_all_keys_for_storage(
        &self,
        module_name: &str,
        storage_name: &str,
        block_hash: &str,
    ) -> anyhow::Result<Vec<String>> {
        let mut all_keys: Vec<String> = Vec::new();
        loop {
            let last = all_keys.last();
            let mut keys: Vec<String> = self
                .ws_client
                .request(
                    "state_getKeysPaged",
                    get_rpc_paged_keys_params(
                        module_name,
                        storage_name,
                        KEY_QUERY_PAGE_SIZE,
                        if let Some(last) = last {
                            Some(last.as_str())
                        } else {
                            None
                        },
                        Some(block_hash),
                    ),
                )
                .await?;
            let keys_length = keys.len();
            all_keys.append(&mut keys);
            if keys_length < KEY_QUERY_PAGE_SIZE {
                break;
            }
        }
        Ok(all_keys)
    }

    pub async fn get_next_session_keys(
        &self,
        validator_map: &mut HashMap<AccountId, ValidatorDetails>,
        block_hash: &str,
    ) -> anyhow::Result<()> {
        log::debug!("Get next session keys.");
        let keys: Vec<String> = validator_map
            .values()
            .map(|validator| {
                get_storage_map_key(&self.metadata, "Session", "NextKeys", &validator.account.id)
            })
            .collect();
        for chunk in keys.chunks(KEY_QUERY_PAGE_SIZE) {
            let chunk_values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                .await?;

            for (storage_key, data) in chunk_values[0].changes.iter() {
                if let Some(data) = data {
                    let account_id = self.account_id_from_storage_key(storage_key);
                    let session_keys = format!("0x{}", hex::encode_upper(&data.0));
                    let validator = validator_map.get_mut(&account_id).unwrap();
                    validator.next_session_keys = session_keys;
                }
            }
        }
        Ok(())
    }

    pub async fn get_queued_session_keys(
        &self,
        validator_map: &mut HashMap<AccountId, ValidatorDetails>,
        block_hash: &str,
    ) -> anyhow::Result<()> {
        log::debug!("Get queued session keys & find out which validators are active next session.");
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Session", "QueuedKeys", Some(block_hash)),
            )
            .await?;
        let maybe_session_key_pairs: anyhow::Result<Vec<(AccountId, [u8; 193])>> =
            decode_hex_string(&hex_string);
        if let Ok(session_key_pairs) = &maybe_session_key_pairs {
            for session_key_pair in session_key_pairs.iter() {
                let session_keys = format!("0x{}", hex::encode_upper(session_key_pair.1));
                if let Some(validator) = validator_map.get_mut(&session_key_pair.0) {
                    validator.is_active_next_session = true;
                    validator.queued_session_keys = Some(session_keys.clone());
                }
            }
        } else {
            let session_key_pairs: Vec<(AccountId, [u8; 225])> =
                decode_hex_string(&hex_string).unwrap();
            for session_key_pair in session_key_pairs.iter() {
                let session_keys = format!("0x{}", hex::encode_upper(session_key_pair.1));
                if let Some(validator) = validator_map.get_mut(&session_key_pair.0) {
                    validator.is_active_next_session = true;
                    validator.queued_session_keys = Some(session_keys.clone());
                }
            }
        }
        Ok(())
    }

    /// Get the complete details of all validators, active and inactive, at the given block.
    #[allow(clippy::cognitive_complexity)]
    pub async fn get_all_validators(
        &self,
        relay_client: &SubstrateClient,
        people_client: &SubstrateClient,
        block_hash: &str,
        era: &Era,
    ) -> anyhow::Result<Vec<ValidatorDetails>> {
        log::info!("Getting all validators.");
        let all_keys: Vec<String> = self
            .get_all_keys_for_storage("Staking", "Validators", block_hash)
            .await?;
        log::info!(
            "There are {} validators (active and waiting).",
            all_keys.len()
        );
        log::debug!("Get complete account, active and para-validator info for all validators.");
        let last_relay_chain_block_number =
            self.get_last_relay_chain_block_number(block_hash).await?;
        let last_relay_chain_block_hash = relay_client
            .get_block_hash(last_relay_chain_block_number as u64)
            .await?;
        let people_finalized_block_hash = people_client.get_finalized_block_hash().await?;
        let mut validator_map: HashMap<AccountId, ValidatorDetails> = HashMap::default();
        {
            let active_validator_account_ids =
                self.get_active_validator_account_ids(block_hash).await?;
            log::debug!("Get para validators and core assignments.");
            let mut para_core_assignment_map: HashMap<AccountId, Option<ParaCoreAssignment>> =
                HashMap::default();
            if let Some(para_validator_indices) = relay_client
                .get_paras_active_validator_indices(&last_relay_chain_block_hash)
                .await?
            {
                let para_validator_index_map = {
                    let mut map: HashMap<u32, AccountId> = HashMap::default();
                    for (para_validator_index, validator_index) in
                        para_validator_indices.iter().enumerate()
                    {
                        if let Some(account_id) =
                            active_validator_account_ids.get(*validator_index as usize)
                        {
                            para_core_assignment_map.insert(*account_id, None);
                            map.insert(para_validator_index as u32, *account_id);
                        }
                    }
                    map
                };
                let para_validator_group_map = {
                    let mut map: HashMap<u32, Vec<AccountId>> = HashMap::default();
                    let para_validator_groups = relay_client
                        .get_para_validator_groups(&last_relay_chain_block_hash)
                        .await?;
                    for (group_index, group) in para_validator_groups.iter().enumerate() {
                        map.insert(
                            group_index as u32,
                            group
                                .iter()
                                .filter_map(|index| para_validator_index_map.get(index))
                                .cloned()
                                .collect(),
                        );
                    }
                    map
                };
                if let Some(para_core_assignments) = relay_client
                    .get_para_core_assignments(&last_relay_chain_block_hash)
                    .await?
                {
                    for assignment in &para_core_assignments {
                        if let Some(group) = para_validator_group_map.get(&assignment.group_index) {
                            for account_id in group {
                                para_core_assignment_map
                                    .insert(*account_id, Some(assignment.clone()));
                            }
                        }
                    }
                }
            };
            log::debug!("Get accounts.");
            let account_ids: Vec<AccountId> = all_keys
                .iter()
                .map(|key| self.account_id_from_storage_key_string(key))
                .collect();
            let accounts = people_client
                .get_accounts(&account_ids, true, people_finalized_block_hash.as_str())
                .await?;
            for account in accounts {
                let is_active = active_validator_account_ids.contains(&account.id);
                let is_para_validator =
                    is_active && para_core_assignment_map.contains_key(&account.id);
                let para_core_assignment = if is_para_validator {
                    para_core_assignment_map.get(&account.id).unwrap().clone()
                } else {
                    None
                };
                validator_map.insert(
                    account.id,
                    ValidatorDetails {
                        account: account.clone(),
                        network_id: self.network_id,
                        is_active,
                        is_para_validator,
                        para_core_assignment,
                        ..Default::default()
                    },
                );
            }
        }
        relay_client
            .get_next_session_keys(&mut validator_map, &last_relay_chain_block_hash)
            .await?;
        relay_client
            .get_queued_session_keys(&mut validator_map, &last_relay_chain_block_hash)
            .await?;
        // get reward destinations
        {
            log::debug!("Get reward destinations.");
            let keys: Vec<String> = validator_map
                .values()
                .map(|validator| {
                    get_storage_map_key(&self.metadata, "Staking", "Payee", &validator.account.id)
                })
                .collect();

            for chunk in keys.chunks(KEY_QUERY_PAGE_SIZE) {
                let chunk_values: Vec<StorageChangeSet<String>> = self
                    .ws_client
                    .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                    .await?;

                for (storage_key, data) in chunk_values[0].changes.iter() {
                    if let Some(data) = data {
                        let account_id = self.account_id_from_storage_key(storage_key);
                        let bytes: &[u8] = &data.0;
                        let reward_destination = RewardDestination::from_bytes(bytes).unwrap();
                        let validator = validator_map.get_mut(&account_id).unwrap();
                        validator.reward_destination = reward_destination;
                    }
                }
            }
        }
        // get nominations
        {
            log::debug!("Get all nominations.");
            let mut all_keys: Vec<String> = Vec::new();
            loop {
                let last = all_keys.last();
                let mut keys: Vec<String> = self
                    .ws_client
                    .request(
                        "state_getKeysPaged",
                        get_rpc_paged_keys_params(
                            "Staking",
                            "Nominators",
                            KEY_QUERY_PAGE_SIZE,
                            if let Some(last) = last {
                                Some(last.as_str())
                            } else {
                                None
                            },
                            Some(block_hash),
                        ),
                    )
                    .await?;
                let keys_length = keys.len();
                all_keys.append(&mut keys);
                if keys_length < KEY_QUERY_PAGE_SIZE {
                    break;
                }
            }

            log::debug!(
                "Got {} nomination storage keys. Accessing storage.",
                all_keys.len()
            );
            let mut nomination_map: HashMap<AccountId, Nomination> = HashMap::default();
            for chunk in all_keys.chunks(KEY_QUERY_PAGE_SIZE) {
                let chunk_values: Vec<StorageChangeSet<String>> = self
                    .ws_client
                    .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                    .await?;
                for (storage_key, data) in chunk_values[0].changes.iter() {
                    if let Some(data) = data {
                        let account_id = self.account_id_from_storage_key(storage_key);
                        let bytes: &[u8] = &data.0;
                        let nomination = Nomination::from_bytes(bytes, account_id).unwrap();
                        nomination_map.insert(nomination.stash_account.id, nomination);
                    }
                }
            }
            log::debug!(
                "Got {} nominations. Get nominator accounts.",
                nomination_map.len()
            );
            // get nominator account details
            {
                let nominator_account_ids: Vec<AccountId> =
                    nomination_map.keys().cloned().collect();
                for account_id_chunk in nominator_account_ids.chunks(KEY_QUERY_PAGE_SIZE) {
                    let accounts = people_client
                        .get_accounts(account_id_chunk, true, people_finalized_block_hash.as_str())
                        .await?;
                    for account in accounts {
                        nomination_map.get_mut(&account.id).unwrap().stash_account =
                            account.clone();
                    }
                }
            }

            log::debug!("Get validator controller account ids.");
            let mut controller_account_id_map: HashMap<AccountId, AccountId> = HashMap::default();
            for validator_account_id in validator_map.keys() {
                controller_account_id_map.insert(*validator_account_id, *validator_account_id);
            }
            log::debug!("Get nomination amounts and self stakes.");
            let controller_account_ids: Vec<AccountId> =
                controller_account_id_map.values().cloned().collect();
            let ledger_storage_keys: Vec<String> = controller_account_ids
                .iter()
                .map(|controller_account_id| {
                    get_storage_map_key(&self.metadata, "Staking", "Ledger", &controller_account_id)
                })
                .collect();
            for chunk in ledger_storage_keys.chunks(KEY_QUERY_PAGE_SIZE) {
                let chunk_values: Vec<StorageChangeSet<String>> = self
                    .ws_client
                    .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                    .await?;
                for (_, data) in chunk_values[0].changes.iter() {
                    if let Some(data) = data {
                        let bytes: &[u8] = &data.0;
                        let stake = Stake::from_bytes(bytes).unwrap();
                        let account_id = &stake.stash_account_id;
                        if let Some(nomination) = nomination_map.get_mut(account_id) {
                            nomination.stake = stake;
                        } else if let Some(validator) = validator_map.get_mut(account_id) {
                            validator.self_stake = stake;
                        }
                    }
                }
            }
            for nomination in nomination_map.values() {
                for account_id in nomination.target_account_ids.iter() {
                    if let Some(validator) = validator_map.get_mut(account_id) {
                        validator.nominations.push(nomination.into());
                        validator.oversubscribed = false;
                    }
                }
            }
            for validator in validator_map.values_mut() {
                validator.nominations.sort_by_key(|nomination| {
                    let mut hasher = FxHasher::default();
                    nomination.stash_account.id.hash(&mut hasher);
                    hasher.finish()
                });
            }
            log::debug!("Nomination data complete.");
        }
        // get validator prefs
        {
            log::debug!("Get validator preferences.");
            let values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request("state_queryStorageAt", rpc_params!(all_keys, &block_hash))
                .await?;
            for (storage_key, data) in values[0].changes.iter() {
                if let Some(data) = data {
                    let mut bytes = &data.0.clone()[..];
                    let preferences = Decode::decode(&mut bytes)?;
                    let validator_account_id = self.account_id_from_storage_key(storage_key);
                    let validator = validator_map.get_mut(&validator_account_id).unwrap();
                    validator.preferences = preferences;
                }
            }
        }
        // get active stakers
        {
            log::debug!("Get active stakers.");
            let era_stakers = self.get_era_stakers(era, block_hash).await?;
            for validator_stake in era_stakers.stakers.iter() {
                if let Some(validator) = validator_map.get_mut(&validator_stake.account.id) {
                    validator.validator_stake = Some(validator_stake.clone());
                }
            }
            // calculate return rates
            let total_staked = self.get_era_total_stake(era.index, block_hash).await?;
            let eras_per_day = 24 * 60 * 60 * 1000
                / get_metadata_era_duration_millis(&relay_client.metadata, &self.metadata)? as u128;
            let last_era_total_reward = self
                .get_era_total_validator_reward(era.index - 1, block_hash)
                .await?;
            let total_return_rate_per_billion =
                (last_era_total_reward * eras_per_day * 365 * 1_000_000_000) / total_staked;
            let average_stake = era_stakers.average_stake();
            for validator in validator_map.values_mut() {
                validator.return_rate_per_billion = if validator.is_active {
                    if validator.validator_stake.is_none() {
                        validator.validator_stake = Some(ValidatorStake {
                            account: validator.account.clone(),
                            self_stake: 0,
                            total_stake: 0,
                            nominators: Vec::new(),
                        });
                        Some(0)
                    } else {
                        let return_rate = (average_stake * total_return_rate_per_billion
                            / validator.validator_stake.as_ref().unwrap().total_stake)
                            * (1_000_000_000
                                - (validator.preferences.commission_per_billion as u128))
                            / 1_000_000_000;
                        Some(return_rate as u32)
                    }
                } else {
                    None
                }
            }
        }
        log::info!("Fetched complete validators data.");
        Ok(validator_map.into_values().collect())
    }

    /// Get the number of all validation intents at the given block.
    pub async fn get_total_validator_count(&self, block_hash: &str) -> anyhow::Result<u32> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Staking", "CounterForValidators", Some(block_hash)),
            )
            .await?;
        decode_hex_string(hex_string.as_str())
    }

    /// Get total rewards earned by validators in the native currency at the given era.
    pub async fn get_era_total_validator_reward(
        &self,
        era_index: u32,
        block_hash: &str,
    ) -> anyhow::Result<Balance> {
        let params = get_rpc_storage_map_params(
            &self.metadata,
            "Staking",
            "ErasValidatorReward",
            &era_index,
            Some(block_hash),
        );
        let hex_string: String = self.ws_client.request("state_getStorage", params).await?;
        decode_hex_string(hex_string.as_str())
    }

    /// Get total amount staked at the given era.
    pub async fn get_era_total_stake(
        &self,
        era_index: u32,
        block_hash: &str,
    ) -> anyhow::Result<Balance> {
        let params = get_rpc_storage_map_params(
            &self.metadata,
            "Staking",
            "ErasTotalStake",
            &era_index,
            Some(block_hash),
        );
        let hex_string: String = self.ws_client.request("state_getStorage", params).await?;
        decode_hex_string(hex_string.as_str())
    }

    async fn get_exposure_metadata_map(
        &self,
        era: &Era,
        block_hash: &str,
    ) -> anyhow::Result<HashMap<AccountId, PagedExposureMetadata<Balance>>> {
        // önce overview'ları çek, sonra her biri için sayfaları çek ve topla
        let mut all_keys: Vec<String> = Vec::new();
        loop {
            let last = all_keys.last();
            let mut keys: Vec<String> = self
                .ws_client
                .request(
                    "state_getKeysPaged",
                    get_rpc_paged_map_keys_params(
                        &self.metadata,
                        "Staking",
                        "ErasStakersOverview",
                        &era.index,
                        KEY_QUERY_PAGE_SIZE,
                        if let Some(last) = last {
                            Some(last.as_str())
                        } else {
                            None
                        },
                        Some(block_hash),
                    ),
                )
                .await?;
            let keys_length = keys.len();
            all_keys.append(&mut keys);
            if keys_length < KEY_QUERY_PAGE_SIZE {
                break;
            }
        }
        let mut exposure_metadata_map: HashMap<AccountId, PagedExposureMetadata<Balance>> =
            HashMap::default();
        for chunk in all_keys.chunks(KEY_QUERY_PAGE_SIZE) {
            let chunk_values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                .await?;

            for (storage_key, data) in chunk_values[0].changes.iter() {
                if let Some(data) = data {
                    let validator_account_id: AccountId = storage_key.0[storage_key.0.len() - 32..]
                        .try_into()
                        .unwrap();
                    let mut bytes: &[u8] = &data.0;
                    exposure_metadata_map.insert(validator_account_id, Decode::decode(&mut bytes)?);
                }
            }
        }
        Ok(exposure_metadata_map)
    }

    /// Get all the active stakes for the given era.
    async fn get_era_stakers_legacy(
        &self,
        era: &Era,
        clipped: bool,
        block_hash: &str,
    ) -> anyhow::Result<EraStakers> {
        let mut all_keys: Vec<String> = Vec::new();
        loop {
            let last = all_keys.last();
            let mut keys: Vec<String> = self
                .ws_client
                .request(
                    "state_getKeysPaged",
                    get_rpc_paged_map_keys_params(
                        &self.metadata,
                        "Staking",
                        if clipped {
                            "ErasStakersClipped"
                        } else {
                            "ErasStakers"
                        },
                        &era.index,
                        KEY_QUERY_PAGE_SIZE,
                        if let Some(last) = last {
                            Some(last.as_str())
                        } else {
                            None
                        },
                        Some(block_hash),
                    ),
                )
                .await?;
            let keys_length = keys.len();
            all_keys.append(&mut keys);
            if keys_length < KEY_QUERY_PAGE_SIZE {
                break;
            }
        }

        let mut stakers: Vec<ValidatorStake> = Vec::new();
        for chunk in all_keys.chunks(KEY_QUERY_PAGE_SIZE) {
            let chunk_values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                .await?;

            for (storage_key, data) in chunk_values[0].changes.iter() {
                if let Some(data) = data {
                    let validator_account_id = self.account_id_from_storage_key(storage_key);
                    let nomination =
                        ValidatorStake::from_bytes_legacy(&data.0, validator_account_id).unwrap();
                    stakers.push(nomination);
                }
            }
        }
        stakers.sort_by_key(|validator_stake| validator_stake.total_stake);
        Ok(EraStakers {
            era: era.clone(),
            stakers,
        })
    }

    /// Get all the active stakes for the given era.
    pub async fn get_era_stakers(&self, era: &Era, block_hash: &str) -> anyhow::Result<EraStakers> {
        let exposure_metadata_map = self.get_exposure_metadata_map(era, block_hash).await?;
        let mut all_keys: Vec<String> = Vec::new();
        loop {
            let last = all_keys.last();
            let mut keys: Vec<String> = self
                .ws_client
                .request(
                    "state_getKeysPaged",
                    get_rpc_paged_map_keys_params(
                        &self.metadata,
                        "Staking",
                        "ErasStakersPaged",
                        &era.index,
                        KEY_QUERY_PAGE_SIZE,
                        if let Some(last) = last {
                            Some(last.as_str())
                        } else {
                            None
                        },
                        Some(block_hash),
                    ),
                )
                .await?;
            let keys_length = keys.len();
            all_keys.append(&mut keys);
            if keys_length < KEY_QUERY_PAGE_SIZE {
                break;
            }
        }
        if all_keys.is_empty() {
            return self.get_era_stakers_legacy(era, true, block_hash).await;
        }

        let mut stakers: Vec<ValidatorStake> = Vec::new();
        for chunk in all_keys.chunks(KEY_QUERY_PAGE_SIZE) {
            let chunk_values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                .await?;

            for (storage_key, data) in chunk_values[0].changes.iter() {
                if let Some(data) = data {
                    let validator_account_id: AccountId = storage_key.0
                        [storage_key.0.len() - (32 + 12)..storage_key.0.len() - 12]
                        .try_into()
                        .unwrap();
                    let validator_exposure =
                        exposure_metadata_map.get(&validator_account_id).unwrap();
                    let nomination = ValidatorStake::from_bytes(
                        &data.0,
                        validator_account_id,
                        validator_exposure.own,
                    )
                    .unwrap();
                    if let Some(index) = stakers
                        .iter()
                        .position(|stake| stake.account.id == validator_account_id)
                    {
                        for nominator in nomination.nominators.iter() {
                            stakers[index].nominators.push(nominator.clone());
                        }
                    } else {
                        stakers.push(nomination);
                    }
                }
            }
        }
        stakers.sort_by_key(|validator_stake| validator_stake.total_stake);
        Ok(EraStakers {
            era: era.clone(),
            stakers,
        })
    }

    /// Get total and individual era reward points earned by validators at the given era.
    /// Will give the points earned so far for an active era.
    pub async fn get_era_reward_points(
        &self,
        era_index: u32,
        block_hash: &str,
    ) -> anyhow::Result<EraRewardPoints> {
        let params = get_rpc_storage_map_params(
            &self.metadata,
            "Staking",
            "ErasRewardPoints",
            &era_index,
            Some(block_hash),
        );
        let maybe_hex_string: Option<String> =
            self.ws_client.request("state_getStorage", params).await?;
        let reward_points = if let Some(hex_string) = maybe_hex_string {
            decode_hex_string(hex_string.as_str())?
        } else {
            Default::default()
        };
        Ok(reward_points)
    }

    /// Get the session index at the given block.
    pub async fn get_current_session_index(&self, block_hash: &str) -> anyhow::Result<u32> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Session", "CurrentIndex", Some(block_hash)),
            )
            .await?;
        decode_hex_string(hex_string.as_str())
    }

    /// Get the complete events in the given block.
    pub async fn get_block_events(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<Result<SubstrateEvent, DecodeError>>> {
        let block = self.get_block(block_hash).await?;
        let mut event_bytes: &[u8] = {
            let events_hex_string: String = self
                .ws_client
                .request(
                    "state_getStorage",
                    get_rpc_storage_plain_params("System", "Events", Some(block_hash)),
                )
                .await?;
            &hex::decode(events_hex_string.trim_start_matches("0x"))?
        };
        SubstrateEvent::decode_events(
            &self.chain,
            &self.metadata,
            self.last_runtime_upgrade_info.spec_version,
            block_hash,
            block,
            &mut event_bytes,
        )
    }

    /// Get the complete extrinsics in the given block.
    pub async fn get_block_extrinsics(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<Result<SubstrateExtrinsic, DecodeError>>> {
        let block = self.get_block(block_hash).await?;
        SubstrateExtrinsic::decode_extrinsics(
            &self.chain,
            self.last_runtime_upgrade_info.spec_version,
            &self.metadata,
            block_hash,
            block,
        )
    }

    /// Get runtime info at the given block.
    pub async fn get_last_runtime_upgrade_info(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<LastRuntimeUpgradeInfo> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("System", "LastRuntimeUpgrade", Some(block_hash)),
            )
            .await?;
        let upgrade_info = LastRuntimeUpgradeInfo::from_substrate_hex_string(hex_string)?;
        Ok(upgrade_info)
    }

    /// Figure the account id of the owner of an imonline key at a given block.
    pub async fn get_im_online_key_owner_account_id(
        &self,
        block_hash: &str,
        im_online_key_hex_string: &str,
    ) -> anyhow::Result<AccountId> {
        let im_online_key_bytes: &[u8] =
            &hex::decode(im_online_key_hex_string.trim_start_matches("0x")).unwrap();
        let params = get_rpc_storage_map_params(
            &self.metadata,
            "Session",
            "KeyOwner",
            &(sp_core::crypto::key_types::IM_ONLINE, im_online_key_bytes),
            Some(block_hash),
        );
        let account_id_hex_string: String =
            self.ws_client.request("state_getStorage", params).await?;
        let account_id = decode_hex_string(&account_id_hex_string)?;
        Ok(account_id)
    }

    /// Get the indices of the paravalidators at the given block.
    pub async fn get_paras_active_validator_indices(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Option<Vec<u32>>> {
        let params =
            get_rpc_storage_plain_params("ParasShared", "ActiveValidatorIndices", Some(block_hash));
        let maybe_indices_vector_hex_string: Option<String> =
            self.ws_client.request("state_getStorage", params).await?;
        if let Some(indices_vector_hex_string) = maybe_indices_vector_hex_string {
            Ok(Some(decode_hex_string(&indices_vector_hex_string)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_last_relay_chain_block_number(&self, block_hash: &str) -> anyhow::Result<u32> {
        let params = get_rpc_storage_plain_params(
            "ParachainSystem",
            "LastRelayChainBlockNumber",
            Some(block_hash),
        );
        let hex_string: String = self.ws_client.request("state_getStorage", params).await?;
        decode_hex_string(&hex_string)
    }

    /// Get parachain validator groups. Indices here are the indices of the result of the
    /// `get_parachain_active_validator_indices` call.
    pub async fn get_para_validator_groups(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<Vec<u32>>> {
        let params =
            get_rpc_storage_plain_params("ParaScheduler", "ValidatorGroups", Some(block_hash));
        let group_double_vector_hex_string: String =
            self.ws_client.request("state_getStorage", params).await?;
        let groups = decode_hex_string(&group_double_vector_hex_string)?;
        Ok(groups)
    }

    pub async fn get_para_core_assignments_legacy(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Option<Vec<ParaCoreAssignment>>> {
        let params = get_rpc_storage_plain_params("ParaInherent", "OnChainVotes", Some(block_hash));
        let maybe_votes_hex_string: Option<String> =
            self.ws_client.request("state_getStorage", params).await?;
        if let Some(hex_string) = maybe_votes_hex_string {
            let votes: ScrapedOnChainVotes = decode_hex_string(&hex_string)?;
            // get availability cores
            let params =
                get_rpc_storage_plain_params("ParaScheduler", "ClaimQueue", Some(block_hash));
            let maybe_cores_hex_string: Option<String> =
                self.ws_client.request("state_getStorage", params).await?;
            if let Some(cores_hex_string) = &maybe_cores_hex_string {
                let cores: Vec<LegacyCoreOccupied> = decode_hex_string(cores_hex_string)?;
                Ok(Some(ParaCoreAssignment::from_on_chain_votes_legacy(
                    3, cores, votes,
                )?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn get_para_core_assignments(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Option<Vec<ParaCoreAssignment>>> {
        let params = get_rpc_storage_plain_params("ParaInherent", "OnChainVotes", Some(block_hash));
        let maybe_votes_hex_string: Option<String> =
            self.ws_client.request("state_getStorage", params).await?;
        if let Some(hex_string) = maybe_votes_hex_string {
            let votes: ScrapedOnChainVotes = decode_hex_string(&hex_string)?;
            let mut group_size: u32 = 0;
            for (_, votes) in votes.backing_validators_per_candidate.iter() {
                group_size = max(group_size, votes.len() as u32);
            }
            // get core claim queue
            let params =
                get_rpc_storage_plain_params("ParaScheduler", "ClaimQueue", Some(block_hash));
            let maybe_cores_hex_string: Option<String> =
                self.ws_client.request("state_getStorage", params).await?;
            if let Some(cores_hex_string) = &maybe_cores_hex_string {
                let claim_queue: BTreeMap<u32, Vec<CoreAssignment>> =
                    decode_hex_string(cores_hex_string)?;
                Ok(Some(ParaCoreAssignment::from_claim_queue(
                    group_size,
                    claim_queue,
                    votes,
                )?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn get_para_votes(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Option<ScrapedOnChainVotes>> {
        let params = get_rpc_storage_plain_params("ParaInherent", "OnChainVotes", Some(block_hash));
        let maybe_votes_hex_string: Option<String> =
            self.ws_client.request("state_getStorage", params).await?;
        if let Some(hex_string) = maybe_votes_hex_string {
            Ok(Some(decode_hex_string(&hex_string)?))
        } else {
            Ok(None)
        }
    }

    /// Validator preferences map at a given block.
    pub async fn get_era_validator_prefs(
        &self,
        era_index: u32,
        block_hash: &str,
    ) -> anyhow::Result<HashMap<AccountId, ValidatorPreferences>> {
        let mut all_keys: Vec<String> = Vec::new();
        loop {
            let last = all_keys.last();
            let mut keys: Vec<String> = self
                .ws_client
                .request(
                    "state_getKeysPaged",
                    get_rpc_paged_map_keys_params(
                        &self.metadata,
                        "Staking",
                        "ErasValidatorPrefs",
                        &era_index,
                        KEY_QUERY_PAGE_SIZE,
                        if let Some(last) = last {
                            Some(last.as_str())
                        } else {
                            None
                        },
                        Some(block_hash),
                    ),
                )
                .await?;
            let keys_length = keys.len();
            all_keys.append(&mut keys);
            if keys_length < KEY_QUERY_PAGE_SIZE {
                break;
            }
        }
        let mut validator_prefs_map: HashMap<AccountId, ValidatorPreferences> = HashMap::default();
        for chunk in all_keys.chunks(KEY_QUERY_PAGE_SIZE) {
            let chunk_values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                .await?;

            for (storage_key, data) in chunk_values[0].changes.iter() {
                if let Some(data) = data {
                    let validator_account_id = self.account_id_from_storage_key(storage_key);
                    let mut bytes: &[u8] = &data.0.clone();
                    let mut bytes_clone: &[u8] = &data.0.clone();
                    let validator_prefs = match Decode::decode(&mut bytes) {
                        Ok(validator_preferences) => validator_preferences,
                        Err(_) => {
                            let legacy_validator_prefs: LegacyValidatorPrefs =
                                Decode::decode(&mut bytes_clone)?;
                            ValidatorPreferences {
                                commission_per_billion: legacy_validator_prefs
                                    .commission
                                    .deconstruct(),
                                blocks_nominations: false,
                            }
                        }
                    };
                    validator_prefs_map.insert(validator_account_id, validator_prefs);
                }
            }
        }
        Ok(validator_prefs_map)
    }

    pub async fn get_conviction_voting_for(
        &self,
        account_id: &AccountId,
        track_id: u16,
        block_hash: Option<&str>,
    ) -> anyhow::Result<
        Option<ConvictionVoting<Balance, AccountId, BlockNumber, u32, ConstU32<{ u32::MAX }>>>,
    > {
        let storage_key = get_storage_double_map_key(
            &self.metadata,
            "ConvictionVoting",
            "VotingFor",
            account_id,
            &track_id,
        );
        let chunk_values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request(
                "state_queryStorageAt",
                rpc_params!(vec![storage_key], block_hash),
            )
            .await?;
        if let Some(value) = chunk_values.first() {
            if let Some((_, Some(data))) = value.changes.first() {
                let mut bytes: &[u8] = &data.0;
                let voting: ConvictionVoting<
                    Balance,
                    AccountId,
                    BlockNumber,
                    u32,
                    ConstU32<{ u32::MAX }>,
                > = Decode::decode(&mut bytes)?;
                return Ok(Some(voting));
            }
        }
        Ok(None)
    }

    pub async fn get_democracy_voting_of(
        &self,
        account_id: &AccountId,
        block_hash: Option<&str>,
    ) -> anyhow::Result<
        Option<DemocracyVoting<Balance, AccountId, BlockNumber, ConstU32<{ u32::MAX }>>>,
    > {
        let storage_key = get_storage_map_key(&self.metadata, "Democracy", "VotingOf", account_id);
        let chunk_values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request(
                "state_queryStorageAt",
                rpc_params!(vec![storage_key], block_hash),
            )
            .await?;
        if let Some(value) = chunk_values.first() {
            if let Some((_, Some(data))) = value.changes.first() {
                let mut bytes: &[u8] = &data.0;
                let voting: DemocracyVoting<
                    Balance,
                    AccountId,
                    BlockNumber,
                    ConstU32<{ u32::MAX }>,
                > = Decode::decode(&mut bytes)?;
                return Ok(Some(voting));
            }
        }
        Ok(None)
    }

    #[async_recursion]
    pub async fn get_account_referendum_vote(
        &self,
        account_id: &AccountId,
        referendum_index: u32,
        block_hash: Option<&'async_recursion str>,
    ) -> anyhow::Result<Option<ReferendumVote>> {
        let maybe_vote = self.get_democracy_voting_of(account_id, block_hash).await?;
        if let Some(vote) = maybe_vote {
            match vote {
                DemocracyVoting::Direct { votes, .. } => {
                    if let Some(referendum_vote) =
                        votes.iter().find(|vote| vote.0 == referendum_index)
                    {
                        let vote = match referendum_vote.1 {
                            AccountVote::Standard { vote, balance } => ReferendumVote {
                                account_id: *account_id,
                                referendum_index,
                                direct_vote: Some(DirectVote {
                                    ty: VoteType::Standard,
                                    aye: if vote.aye { Some(balance) } else { None },
                                    nay: if !vote.aye { Some(balance) } else { None },
                                    abstain: None,
                                    conviction: None,
                                }),
                                delegated_vote: None,
                            },
                            AccountVote::Split { aye, nay } => ReferendumVote {
                                account_id: *account_id,
                                referendum_index,
                                direct_vote: Some(DirectVote {
                                    ty: VoteType::Split,
                                    aye: Some(aye),
                                    nay: Some(nay),
                                    abstain: None,
                                    conviction: None,
                                }),
                                delegated_vote: None,
                            },
                        };
                        return Ok(Some(vote));
                    }
                }
                DemocracyVoting::Delegating {
                    balance, target, ..
                } => {
                    if let Some(delegate_vote) = self
                        .get_account_referendum_vote(&target, referendum_index, block_hash)
                        .await?
                    {
                        if let Some(delegate_direct_vote) = delegate_vote.direct_vote {
                            let vote = ReferendumVote {
                                account_id: *account_id,
                                referendum_index,
                                direct_vote: None,
                                delegated_vote: Some(DelegatedVote {
                                    target_account_id: target,
                                    balance,
                                    conviction: 0,
                                    delegate_account_id: target,
                                    vote: delegate_direct_vote,
                                }),
                            };
                            return Ok(Some(vote));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    #[async_recursion]
    pub async fn get_account_referendum_conviction_vote(
        &self,
        account_id: &AccountId,
        track_id: u16,
        referendum_id: u32,
        block_hash: Option<&'async_recursion str>,
    ) -> anyhow::Result<Option<ReferendumVote>> {
        let maybe_vote = self
            .get_conviction_voting_for(account_id, track_id, block_hash)
            .await?;
        if let Some(vote) = maybe_vote {
            match vote {
                ConvictionVoting::Casting(casting) => {
                    if let Some(referendum_vote) = casting
                        .votes
                        .iter()
                        .find(|vote| vote.0 == referendum_id)
                        .map(|vote| vote.1)
                    {
                        let vote = match referendum_vote {
                            ConvictionVote::Standard { vote, balance } => ReferendumVote {
                                account_id: *account_id,
                                referendum_index: referendum_id,
                                direct_vote: Some(DirectVote {
                                    ty: VoteType::Standard,
                                    aye: if vote.aye { Some(balance) } else { None },
                                    nay: if !vote.aye { Some(balance) } else { None },
                                    abstain: None,
                                    conviction: Some(get_democracy_conviction_u8(&vote.conviction)),
                                }),
                                delegated_vote: None,
                            },
                            ConvictionVote::Split { aye, nay } => ReferendumVote {
                                account_id: *account_id,
                                referendum_index: referendum_id,
                                direct_vote: Some(DirectVote {
                                    ty: VoteType::Split,
                                    aye: Some(aye),
                                    nay: Some(nay),
                                    abstain: None,
                                    conviction: None,
                                }),
                                delegated_vote: None,
                            },
                            ConvictionVote::SplitAbstain { aye, nay, abstain } => ReferendumVote {
                                account_id: *account_id,
                                referendum_index: referendum_id,
                                direct_vote: Some(DirectVote {
                                    ty: VoteType::SplitAbstain,
                                    aye: Some(aye),
                                    nay: Some(nay),
                                    abstain: Some(abstain),
                                    conviction: None,
                                }),
                                delegated_vote: None,
                            },
                        };
                        return Ok(Some(vote));
                    }
                }
                ConvictionVoting::Delegating(delegating) => {
                    if let Some(delegate_vote) = self
                        .get_account_referendum_conviction_vote(
                            &delegating.target,
                            track_id,
                            referendum_id,
                            block_hash,
                        )
                        .await?
                    {
                        if let Some(delegate_direct_vote) = delegate_vote.direct_vote {
                            let vote = ReferendumVote {
                                account_id: *account_id,
                                referendum_index: referendum_id,
                                direct_vote: None,
                                delegated_vote: Some(DelegatedVote {
                                    target_account_id: delegating.target,
                                    balance: delegating.balance,
                                    conviction: get_democracy_conviction_u8(&delegating.conviction),
                                    delegate_account_id: delegating.target,
                                    vote: delegate_direct_vote,
                                }),
                            };
                            return Ok(Some(vote));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    async fn subscribe_to_blocks<F>(
        &self,
        subscribe_method_name: &str,
        unsubscribe_method_name: &str,
        timeout_seconds: u64,
        callback: impl Fn(BlockHeader) -> F,
    ) where
        F: Future<Output = anyhow::Result<()>>,
    {
        let mut subscription: Subscription<BlockHeader> = match self
            .ws_client
            .subscribe(
                subscribe_method_name,
                rpc_params!(),
                unsubscribe_method_name,
            )
            .await
        {
            Ok(subscription) => subscription,
            Err(error) => {
                log::error!("Error while subscribing to blocks: {error:?}");
                return;
            }
        };

        while let Ok(maybe_block_header_result) = timeout(
            std::time::Duration::from_secs(timeout_seconds),
            subscription.next(),
        )
        .await
        {
            match maybe_block_header_result {
                Some(block_header_result) => match block_header_result {
                    Ok(block_header) => {
                        if let Err(error) = callback(block_header).await {
                            log::error!("Error in callback: {error:?}");
                            break;
                        }
                    }
                    Err(error) => {
                        log::error!("Error while getting block header: {error:?}");
                        log::error!("Will exit new block subscription.");
                        break;
                    }
                },
                None => {
                    log::error!("Empty block header. Will exit new block subscription.");
                    break;
                }
            }
        }
    }

    /// Subscribes to new blocks.
    pub async fn subscribe_to_new_blocks<F>(
        &self,
        timeout_seconds: u64,
        callback: impl Fn(BlockHeader) -> F,
    ) where
        F: Future<Output = anyhow::Result<()>>,
    {
        self.subscribe_to_blocks(
            "chain_subscribeNewHeads",
            "chain_unsubscribeNewHeads",
            timeout_seconds,
            callback,
        )
        .await;
    }

    /// Subscribes to finalized blocks.
    pub async fn subscribe_to_finalized_blocks<F>(
        &self,
        timeout_seconds: u64,
        callback: impl Fn(BlockHeader) -> F,
    ) where
        F: Future<Output = anyhow::Result<()>>,
    {
        self.subscribe_to_blocks(
            "chain_subscribeFinalizedHeads",
            "chain_unsubscribeFinalizedHeads",
            timeout_seconds,
            callback,
        )
        .await;
    }
}
