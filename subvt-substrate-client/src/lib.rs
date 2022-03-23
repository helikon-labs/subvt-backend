//! SubVT Substrate client implementation.
use crate::storage_utility::{
    get_rpc_paged_keys_params, get_rpc_paged_map_keys_params, get_rpc_storage_map_params,
    get_rpc_storage_plain_params, get_storage_map_key,
};
use jsonrpsee::{
    core::client::{Client, ClientT, Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
use log::{debug, error, trace};
use parity_scale_codec::Decode;
use sp_core::storage::{StorageChangeSet, StorageKey};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::convert::TryInto;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use subvt_config::Config;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::paras::ParaCoreAssignment;
use subvt_types::substrate::{
    event::SubstrateEvent, extrinsic::SubstrateExtrinsic, legacy::LegacyValidatorPrefs,
    metadata::Metadata, Account, Balance, Block, BlockHeader, BlockWrapper, Chain, Epoch, Era,
    EraRewardPoints, EraStakers, IdentityRegistration, LastRuntimeUpgradeInfo, Nomination,
    RewardDestination, Stake, SuperAccountId, SystemProperties, ValidatorPreferences,
    ValidatorStake,
};
/// Substrate client structure and its functions.
/// This is the main gateway for SubVT to a Substrate node RPC interface.
use subvt_types::subvt::ValidatorDetails;
use subvt_utility::decode_hex_string;
use tokio::time::timeout;

mod storage_utility;

const KEY_QUERY_PAGE_SIZE: usize = 1000;

/// The client.
pub struct SubstrateClient {
    pub chain: Chain,
    pub metadata: Metadata,
    pub system_properties: SystemProperties,
    ws_client: Client,
}

impl SubstrateClient {
    /// Connect to the node and construct a new Substrate client.
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        debug!("Constructing Substrate client.");
        let ws_client = WsClientBuilder::default()
            .connection_timeout(std::time::Duration::from_secs(
                config.substrate.connection_timeout_seconds,
            ))
            .request_timeout(std::time::Duration::from_secs(
                config.substrate.request_timeout_seconds,
            ))
            .build(&config.substrate.rpc_url)
            .await?;
        debug!("Substrate connection successful.");
        // get current block hash
        let block_hash: String = ws_client.request("chain_getBlockHash", None).await?;
        let chain: String = ws_client.request("system_chain", None).await?;
        let chain = Chain::from_str(chain.as_str())?;
        let mut metadata = {
            let metadata_response: String = ws_client
                .request("state_getMetadata", rpc_params!(&block_hash))
                .await?;
            Metadata::from(metadata_response.as_str())?
        };
        debug!("Got metadata.");
        // metadata.log_all_calls();
        // metadata.log_all_events();
        metadata.check_primitive_argument_support(&chain)?;
        let last_runtime_upgrade_hex_string: String = ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("System", "LastRuntimeUpgrade", Some(&block_hash)),
            )
            .await?;
        metadata.last_runtime_upgrade_info =
            LastRuntimeUpgradeInfo::from_substrate_hex_string(last_runtime_upgrade_hex_string)?;
        debug!("Got last runtime upgrade info.");
        let system_properties: SystemProperties =
            ws_client.request("system_properties", None).await?;
        debug!("Got system properties. {:?}", system_properties);
        Ok(Self {
            chain,
            metadata,
            system_properties,
            ws_client,
        })
    }

    pub async fn set_metadata_at_block(&mut self, block_hash: &str) -> anyhow::Result<()> {
        let mut metadata = {
            let metadata_response: String = self
                .ws_client
                .request("state_getMetadata", rpc_params!(&block_hash))
                .await?;
            Metadata::from(metadata_response.as_str())?
        };
        // metadata.log_all_calls();
        // metadata.log_all_events();
        metadata.check_primitive_argument_support(&self.chain)?;
        metadata.last_runtime_upgrade_info = self.get_last_runtime_upgrade_info(block_hash).await?;
        self.metadata = metadata;
        Ok(())
    }

    pub async fn get_current_block_hash(&self) -> anyhow::Result<String> {
        let hash = self.ws_client.request("chain_getBlockHash", None).await?;
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
            .request("chain_getFinalizedHead", None)
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

    /// Get active era at the given block.
    pub async fn get_active_era(&self, block_hash: &str) -> anyhow::Result<Era> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Staking", "ActiveEra", Some(block_hash)),
            )
            .await?;
        let active_era_info = Era::from(
            hex_string.as_str(),
            self.metadata.constants.era_duration_millis,
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
    pub async fn get_current_epoch(&self, block_hash: &str) -> anyhow::Result<Epoch> {
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
        let start_timestamp: u64 = {
            let hex_string: String = self
                .ws_client
                .request(
                    "state_getStorage",
                    get_rpc_storage_plain_params(
                        "Timestamp",
                        "Now",
                        Some(start_block_hash.as_str()),
                    ),
                )
                .await?;
            decode_hex_string(hex_string.as_str())?
        };
        let end_timestamp = start_timestamp + self.metadata.constants.epoch_duration_millis;
        Ok(Epoch {
            index,
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
        block_hash: &str,
    ) -> anyhow::Result<Option<AccountId>> {
        let storage_key =
            get_storage_map_key(&self.metadata, "Staking", "Bonded", stash_account_id);
        let chunk_values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request(
                "state_queryStorageAt",
                rpc_params!(vec![storage_key], block_hash),
            )
            .await?;
        if let Some(value) = chunk_values.get(0) {
            if let Some((_, Some(data))) = value.changes.get(0) {
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
        block_hash: &str,
    ) -> anyhow::Result<Option<Stake>> {
        let storage_key =
            get_storage_map_key(&self.metadata, "Staking", "Ledger", controller_account_id);
        let chunk_values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request(
                "state_queryStorageAt",
                rpc_params!(vec![storage_key], block_hash),
            )
            .await?;
        if let Some(value) = chunk_values.get(0) {
            if let Some((_, Some(data))) = value.changes.get(0) {
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
        block_hash: &str,
    ) -> anyhow::Result<Option<AccountId>> {
        match self.get_stake(controller_account_id, block_hash).await? {
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
        let mut map = HashMap::new();
        let keys: Vec<String> = account_ids
            .iter()
            .map(|account_id| get_storage_map_key(&self.metadata, "Staking", "Bonded", account_id))
            .collect();
        if keys.is_empty() {
            return Ok(HashMap::new());
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
        let keys: Vec<String> = account_ids
            .iter()
            .map(|account_id| {
                get_storage_map_key(&self.metadata, "Identity", "SuperOf", &account_id)
            })
            .collect();
        trace!("Got {} keys for super accounts.", keys.len());
        if keys.is_empty() {
            return Ok(HashMap::new());
        }
        let values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request("state_queryStorageAt", rpc_params!(keys, &block_hash))
            .await?;
        trace!(
            "Got {} optional super accounts records.",
            values[0].changes.len()
        );
        let mut parent_account_map: HashMap<AccountId, (AccountId, Option<String>)> =
            HashMap::new();
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
        trace!(
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
        trace!("Got {} storage keys for identities.", keys.len());
        if keys.is_empty() {
            return Ok(HashMap::new());
        }
        let values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request("state_queryStorageAt", rpc_params!(keys, &block_hash))
            .await?;
        trace!("Got {} optional identities.", values[0].changes.len());
        let mut identity_map: HashMap<AccountId, IdentityRegistration> = HashMap::new();
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
                    account.child_display = parent_account_id.1.clone();
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

    /// Get the complete details of all validators, active and inactive, at the given block.
    pub async fn get_all_validators(
        &self,
        block_hash: &str,
        era: &Era,
    ) -> anyhow::Result<Vec<ValidatorDetails>> {
        debug!("Getting all validators...");
        let max_nominator_rewarded_per_validator: u32 = self
            .metadata
            .module("Staking")?
            .constant("MaxNominatorRewardedPerValidator")?
            .value()?;
        let all_keys: Vec<String> = self
            .get_all_keys_for_storage("Staking", "Validators", block_hash)
            .await?;
        debug!(
            "There are {} validators (active and waiting).",
            all_keys.len()
        );
        debug!("Get complete account, active and para-validator info for all validators.");
        let mut validator_map: HashMap<AccountId, ValidatorDetails> = HashMap::new();
        {
            let active_validator_account_ids =
                self.get_active_validator_account_ids(block_hash).await?;
            debug!("Get para validators and core assignments.");
            let mut para_core_assignment_map: HashMap<AccountId, Option<ParaCoreAssignment>> =
                HashMap::new();
            if let Some(para_validator_indices) =
                self.get_paras_active_validator_indices(block_hash).await?
            {
                let para_validator_index_map = {
                    let mut map: HashMap<u32, AccountId> = HashMap::new();
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
                    let mut map: HashMap<u32, Vec<AccountId>> = HashMap::new();
                    let para_validator_groups = self.get_para_validator_groups(block_hash).await?;
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
                let para_core_assignments = self.get_para_core_assignments(block_hash).await?;
                for assignment in &para_core_assignments {
                    if let Some(group) = para_validator_group_map.get(&assignment.group_index) {
                        for account_id in group {
                            para_core_assignment_map.insert(*account_id, Some(assignment.clone()));
                        }
                    }
                }
            };
            debug!("Get accounts.");
            let account_ids: Vec<AccountId> = all_keys
                .iter()
                .map(|key| self.account_id_from_storage_key_string(key))
                .collect();
            let accounts = self.get_accounts(&account_ids, block_hash).await?;
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
                        is_active,
                        is_para_validator,
                        para_core_assignment,
                        ..Default::default()
                    },
                );
            }
        }
        // get next session keys
        {
            debug!("Get session keys.");
            let keys: Vec<String> = validator_map
                .values()
                .map(|validator| {
                    get_storage_map_key(
                        &self.metadata,
                        "Session",
                        "NextKeys",
                        &validator.account.id,
                    )
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
        }
        // get next session active validator keys
        {
            debug!("Find out which validators are active next session.");
            let hex_string: String = self
                .ws_client
                .request(
                    "state_getStorage",
                    get_rpc_storage_plain_params("Session", "QueuedKeys", Some(block_hash)),
                )
                .await?;
            let session_key_pairs: Vec<(AccountId, [u8; 192])> =
                decode_hex_string(&hex_string).unwrap();
            for session_key_pair in session_key_pairs.iter() {
                let session_keys = format!("0x{}", hex::encode_upper(session_key_pair.1));
                if let Some(validator) = validator_map.get_mut(&session_key_pair.0) {
                    validator.active_next_session = validator.next_session_keys == session_keys;
                }
            }
        }
        // get reward destinations
        {
            debug!("Get reward destinations.");
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
            debug!("Get all nominations.");
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

            debug!(
                "Got {} nomination storage keys. Accessing storage.",
                all_keys.len()
            );
            let mut nomination_map: HashMap<AccountId, Nomination> = HashMap::new();
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
            debug!(
                "Got {} nominations. Get nominator accounts.",
                nomination_map.len()
            );
            // get nominator account details
            {
                let nominator_account_ids: Vec<AccountId> =
                    nomination_map.keys().cloned().collect();
                for account_id_chunk in nominator_account_ids.chunks(KEY_QUERY_PAGE_SIZE) {
                    let accounts = self.get_accounts(account_id_chunk, block_hash).await?;
                    for account in accounts {
                        nomination_map.get_mut(&account.id).unwrap().stash_account =
                            account.clone();
                    }
                }
            }

            debug!("Get validator controller account ids.");
            let mut controller_storage_keys: Vec<String> = validator_map
                .keys()
                .map(|validator_account_id| {
                    get_storage_map_key(&self.metadata, "Staking", "Bonded", validator_account_id)
                })
                .collect();
            for nominator_stash_account_id in nomination_map.keys() {
                controller_storage_keys.push(get_storage_map_key(
                    &self.metadata,
                    "Staking",
                    "Bonded",
                    nominator_stash_account_id,
                ));
            }
            let mut controller_account_id_map: HashMap<AccountId, AccountId> = HashMap::new();
            for chunk in controller_storage_keys.chunks(KEY_QUERY_PAGE_SIZE) {
                let chunk_values: Vec<StorageChangeSet<String>> = self
                    .ws_client
                    .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                    .await?;
                for (storage_key, data) in chunk_values[0].changes.iter() {
                    if let Some(data) = data {
                        let account_id = self.account_id_from_storage_key(storage_key);
                        let mut bytes: &[u8] = &data.0;
                        let controller_account_id: AccountId = Decode::decode(&mut bytes).unwrap();
                        if let Some(validator) = validator_map.get_mut(&account_id) {
                            validator.controller_account_id = controller_account_id;
                        }
                        controller_account_id_map.insert(account_id, controller_account_id);
                    }
                }
            }
            debug!("Get nomination amounts and self stakes.");
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
                        let stake: Stake = Stake::from_bytes(bytes).unwrap();
                        let account_id = &stake.stash_account_id;
                        if let Some(nomination) = nomination_map.get_mut(account_id) {
                            nomination.stake = stake;
                        } else {
                            let validator = validator_map.get_mut(account_id).unwrap();
                            validator.self_stake = stake;
                        }
                    }
                }
            }
            for nomination in nomination_map.values() {
                for account_id in nomination.target_account_ids.iter() {
                    if let Some(validator) = validator_map.get_mut(account_id) {
                        validator.nominations.push(nomination.clone());
                        validator.oversubscribed = validator.nominations.len()
                            > max_nominator_rewarded_per_validator as usize;
                    }
                }
            }
            for validator in validator_map.values_mut() {
                validator.nominations.sort_by_key(|nomination| {
                    let mut hasher = DefaultHasher::new();
                    nomination.stash_account.id.hash(&mut hasher);
                    hasher.finish()
                });
            }
            debug!("Nomination data complete.");
        }
        // get validator prefs
        {
            debug!("Get validator preferences.");
            let values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request("state_queryStorageAt", rpc_params!(all_keys, &block_hash))
                .await?;
            for (storage_key, data) in values[0].changes.iter() {
                if let Some(data) = data {
                    let bytes: &[u8] = &data.0;
                    let preferences = ValidatorPreferences::from_bytes(bytes).unwrap();
                    let validator_account_id = self.account_id_from_storage_key(storage_key);
                    let validator = validator_map.get_mut(&validator_account_id).unwrap();
                    validator.preferences = preferences;
                }
            }
        }
        // get active stakers
        {
            debug!("Get active stakers.");
            let era_stakers = self.get_era_stakers(era, true, block_hash).await?;
            for validator_stake in &era_stakers.stakers {
                if let Some(validator) = validator_map.get_mut(&validator_stake.account.id) {
                    validator.validator_stake = Some(validator_stake.clone());
                }
            }
            // calculate return rates
            let total_staked = self.get_era_total_stake(era.index, block_hash).await?;
            let eras_per_day =
                (24 * 60 * 60 * 1000 / self.metadata.constants.era_duration_millis) as u128;
            let last_era_total_reward = self
                .get_era_total_validator_reward(era.index - 1, block_hash)
                .await?;
            let total_return_rate_per_billion =
                (last_era_total_reward * eras_per_day * 365 * 1_000_000_000) / total_staked;
            let average_stake = era_stakers.average_stake();
            for validator in validator_map.values_mut() {
                validator.return_rate_per_billion = if validator.is_active {
                    let return_rate = (average_stake * total_return_rate_per_billion
                        / validator.validator_stake.as_ref().unwrap().total_stake)
                        * (1_000_000_000 - (validator.preferences.commission_per_billion as u128))
                        / 1_000_000_000;
                    Some(return_rate as u32)
                } else {
                    None
                }
            }
        }
        debug!("Validator data complete.");
        Ok(validator_map
            .into_iter()
            .map(|(_, validator)| validator)
            .collect())
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

    /// Get all the active stakes for the given era.
    pub async fn get_era_stakers(
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
                        ValidatorStake::from_bytes(&data.0, validator_account_id).unwrap();
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
        let hex_string: String = self.ws_client.request("state_getStorage", params).await?;
        let reward_points = decode_hex_string(hex_string.as_str())?;
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
    pub async fn get_block_events(&self, block_hash: &str) -> anyhow::Result<Vec<SubstrateEvent>> {
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
        SubstrateEvent::decode_events(&self.chain, &self.metadata, block, &mut event_bytes)
    }

    /// Get the complete extrinsics in the given block.
    pub async fn get_block_extrinsics(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<SubstrateExtrinsic>> {
        let block = self.get_block(block_hash).await?;
        SubstrateExtrinsic::decode_extrinsics(&self.chain, &self.metadata, block)
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

    pub async fn get_para_core_assignments(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<ParaCoreAssignment>> {
        let params = get_rpc_storage_plain_params("ParaScheduler", "Scheduled", Some(block_hash));
        let availability_core_vector_hex_string: String =
            self.ws_client.request("state_getStorage", params).await?;
        let assignments = ParaCoreAssignment::from_core_assignment_vector_hex_string(
            &availability_core_vector_hex_string,
        )?;
        Ok(assignments)
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
        let mut validator_prefs_map: HashMap<AccountId, ValidatorPreferences> = HashMap::new();
        for chunk in all_keys.chunks(KEY_QUERY_PAGE_SIZE) {
            let chunk_values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request("state_queryStorageAt", rpc_params!(chunk, &block_hash))
                .await?;

            for (storage_key, data) in chunk_values[0].changes.iter() {
                if let Some(data) = data {
                    let validator_account_id = self.account_id_from_storage_key(storage_key);
                    let bytes: &[u8] = &data.0;
                    let mut bytes_clone: &[u8] = &data.0.clone();
                    let validator_prefs = match ValidatorPreferences::from_bytes(bytes) {
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
            .subscribe(subscribe_method_name, None, unsubscribe_method_name)
            .await
        {
            Ok(subscription) => subscription,
            Err(error) => {
                error!("Error while subscribing to blocks: {:?}", error);
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
                            error!("Error in callback: {:?}", error);
                            break;
                        }
                    }
                    Err(error) => {
                        error!("Error while getting block header: {:?}", error);
                        error!("Will exit new block subscription.");
                        break;
                    }
                },
                None => {
                    error!("Empty block header. Will exit new block subscription.");
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
