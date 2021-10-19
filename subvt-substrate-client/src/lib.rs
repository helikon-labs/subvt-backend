use crate::storage_utility::{
    get_rpc_paged_keys_params, get_rpc_paged_map_keys_params, get_rpc_storage_map_params,
    get_rpc_storage_plain_params, get_storage_map_key,
};
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use jsonrpsee_types::{
    traits::{Client, SubscriptionClient},
    v2::params::JsonRpcParams,
    Subscription,
};
use log::{debug, error};
use parity_scale_codec::Decode;
use sp_core::storage::{StorageChangeSet, StorageKey};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use subvt_config::Config;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::{
    event::SubstrateEvent, extrinsic::SubstrateExtrinsic, metadata::Metadata, Account, Balance,
    Block, BlockHeader, BlockWrapper, Chain, Epoch, Era, EraRewardPoints, EraStakers,
    IdentityRegistration, LastRuntimeUpgradeInfo, Nomination, RewardDestination, Stake,
    SuperAccountId, SystemProperties, ValidatorPreferences, ValidatorStake,
};
/// Substrate client structure and its functions.
/// This is the main gateway to a Substrate node through its RPC interface.
use subvt_types::subvt::InactiveValidator;
use subvt_utility::decode_hex_string;

mod storage_utility;

const KEY_QUERY_PAGE_SIZE: usize = 1000;

/// The client.
pub struct SubstrateClient {
    pub chain: Chain,
    pub metadata: Metadata,
    pub system_properties: SystemProperties,
    ws_client: WsClient,
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
        let block_hash: String = ws_client
            .request("chain_getBlockHash", JsonRpcParams::NoParams)
            .await?;
        let chain: String = ws_client
            .request("system_chain", JsonRpcParams::NoParams)
            .await?;
        let chain = Chain::from_str(chain.as_str())?;
        let mut metadata = {
            let metadata_response: String = ws_client
                .request(
                    "state_getMetadata",
                    JsonRpcParams::Array(vec![block_hash.clone().into()]),
                )
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
        let system_properties: SystemProperties = ws_client
            .request("system_properties", JsonRpcParams::NoParams)
            .await?;
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
                .request(
                    "state_getMetadata",
                    JsonRpcParams::Array(vec![block_hash.into()]),
                )
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
        let hash = self
            .ws_client
            .request("chain_getBlockHash", JsonRpcParams::NoParams)
            .await?;
        Ok(hash)
    }

    /// Get a block hash by its number.
    pub async fn get_block_hash(&self, block_number: u64) -> anyhow::Result<String> {
        let hash = self
            .ws_client
            .request(
                "chain_getBlockHash",
                JsonRpcParams::Array(vec![block_number.into()]),
            )
            .await?;
        Ok(hash)
    }

    /// Get a block header by its hash.
    pub async fn get_block_header(&self, block_hash: &str) -> anyhow::Result<BlockHeader> {
        let header = self
            .ws_client
            .request(
                "chain_getHeader",
                JsonRpcParams::Array(vec![block_hash.into()]),
            )
            .await?;
        Ok(header)
    }

    /// Get the hash of the current finalized block.
    pub async fn get_finalized_block_hash(&self) -> anyhow::Result<String> {
        let hash: String = self
            .ws_client
            .request("chain_getFinalizedHead", JsonRpcParams::NoParams)
            .await?;
        Ok(hash)
    }

    /// Get a block.
    async fn get_block(&self, block_hash: &str) -> anyhow::Result<Block> {
        let block_wrapper: BlockWrapper = self
            .ws_client
            .request(
                "chain_getBlock",
                JsonRpcParams::Array(vec![block_hash.into()]),
            )
            .await?;
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

    pub async fn get_current_epoch_index(&self, block_hash: &str) -> anyhow::Result<u64> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Babe", "EpochIndex", Some(block_hash)),
            )
            .await?;
        Ok(decode_hex_string(hex_string.as_str())?)
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
        let start_timestamp_millis: u64 = {
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
        let start_timestamp = start_timestamp_millis / 1000;
        let end_timestamp_millis =
            start_timestamp_millis + self.metadata.constants.epoch_duration_millis;
        let end_timestamp = end_timestamp_millis / 1000;
        Ok(Epoch {
            index,
            start_block_number,
            start_timestamp,
            end_timestamp,
        })
    }

    fn account_id_from_storage_key_string(&self, storage_key_string: &str) -> AccountId {
        let hex_string = &storage_key_string[(storage_key_string.len() - 64)..];
        decode_hex_string(hex_string).unwrap()
    }

    fn account_id_from_storage_key(&self, storage_key: &StorageKey) -> AccountId {
        storage_key.0[storage_key.0.len() - 32..]
            .try_into()
            .unwrap()
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

    /// Get the list of all active validators at the given block.
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
        Ok(decode_hex_string(hex_string.as_str())?)
    }

    pub async fn get_parent_account_ids(
        &self,
        account_ids: &[AccountId],
        block_hash: &str,
    ) -> anyhow::Result<HashMap<AccountId, AccountId>> {
        let keys: Vec<String> = account_ids
            .iter()
            .map(|account_id| {
                get_storage_map_key(&self.metadata, "Identity", "SuperOf", &account_id)
            })
            .collect();
        debug!("Got {} keys for super accounts.", keys.len());
        if keys.is_empty() {
            return Ok(HashMap::new());
        }
        let values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request(
                "state_queryStorageAt",
                JsonRpcParams::Array(vec![keys.into(), block_hash.into()]),
            )
            .await?;
        debug!(
            "Got {} optional super accounts records.",
            values[0].changes.len()
        );
        let mut parent_account_map: HashMap<AccountId, AccountId> = HashMap::new();
        for (storage_key, storage_data) in values[0].changes.iter() {
            if let Some(data) = storage_data {
                let account_id = self.account_id_from_storage_key(storage_key);
                let mut bytes: &[u8] = &data.0;
                let super_identity: SuperAccountId = Decode::decode(&mut bytes).unwrap();
                parent_account_map.insert(account_id, super_identity.0);
            }
        }
        debug!(
            "Got {} super accounts. Get identities for super accounts.",
            parent_account_map.len()
        );
        Ok(parent_account_map)
    }

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
        debug!("Got {} storage keys for identities.", keys.len());
        if keys.is_empty() {
            return Ok(HashMap::new());
        }
        let values: Vec<StorageChangeSet<String>> = self
            .ws_client
            .request(
                "state_queryStorageAt",
                JsonRpcParams::Array(vec![keys.into(), block_hash.into()]),
            )
            .await?;
        debug!("Got {} optional identities.", values[0].changes.len());
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

    pub async fn get_accounts(
        &self,
        account_ids: &[AccountId],
        block_hash: &str,
    ) -> anyhow::Result<Vec<Account>> {
        let identity_map = { self.get_identities(account_ids, block_hash).await? };
        let parent_account_id_map = { self.get_parent_account_ids(account_ids, block_hash).await? };
        let parent_account_identity_map = {
            let super_account_ids: Vec<AccountId> =
                parent_account_id_map.values().cloned().collect();
            self.get_identities(&super_account_ids, block_hash).await?
        };
        let accounts: Vec<Account> = account_ids
            .iter()
            .cloned()
            .map(|account_id| {
                let mut account = Account {
                    id: account_id.clone(),
                    ..Default::default()
                };
                if let Some(identity) = identity_map.get(&account_id) {
                    account.identity = Some(identity.clone());
                }
                if let Some(parent_account_id) = parent_account_id_map.get(&account_id) {
                    let mut parent_account = Account {
                        id: parent_account_id.clone(),
                        ..Default::default()
                    };
                    if let Some(parent_account_identity) =
                        parent_account_identity_map.get(parent_account_id)
                    {
                        parent_account.identity = Some(parent_account_identity.clone());
                    }
                    account.parent = Box::new(Some(parent_account));
                }
                account
            })
            .collect();
        Ok(accounts)
    }

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

    /// Get the list of all inactive validators at the given block.
    pub async fn get_all_inactive_validators(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<InactiveValidator>> {
        debug!("Getting all inactive validators...");
        let active_validator_account_ids =
            self.get_active_validator_account_ids(block_hash).await?;
        let max_nominator_rewarded_per_validator: u32 = self
            .metadata
            .module("Staking")?
            .constant("MaxNominatorRewardedPerValidator")?
            .value()?;
        let all_keys: Vec<String> = self
            .get_all_keys_for_storage("Staking", "Validators", block_hash)
            .await?
            .into_iter()
            .filter(|key| {
                !active_validator_account_ids
                    .contains(&self.account_id_from_storage_key_string(key))
            })
            .collect();

        let mut inactive_validator_map: HashMap<AccountId, InactiveValidator> = HashMap::new();
        {
            let account_ids: Vec<AccountId> = all_keys
                .iter()
                .map(|key| self.account_id_from_storage_key_string(key))
                .collect();
            let accounts = self.get_accounts(&account_ids, block_hash).await?;
            for account in accounts {
                inactive_validator_map.insert(
                    account.id.clone(),
                    InactiveValidator {
                        account: account.clone(),
                        ..Default::default()
                    },
                );
            }
        }
        debug!(
            "There are {} inactive validators.",
            inactive_validator_map.len()
        );
        // get next session keys
        {
            debug!("Get next session keys for all validators.");
            let keys: Vec<String> = inactive_validator_map
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
                    .request(
                        "state_queryStorageAt",
                        JsonRpcParams::Array(vec![chunk.into(), block_hash.into()]),
                    )
                    .await?;

                for (storage_key, data) in chunk_values[0].changes.iter() {
                    if let Some(data) = data {
                        let account_id = self.account_id_from_storage_key(storage_key);
                        let session_keys = format!("0x{}", hex::encode(&data.0));
                        let validator = inactive_validator_map.get_mut(&account_id).unwrap();
                        validator.next_session_keys = session_keys;
                    }
                }
            }
            debug!("Got next session keys.");
        }
        // get next session active validator keys
        {
            debug!("Get queued keys for the next session.");
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
                let session_keys = format!("0x{}", hex::encode(session_key_pair.1));
                if let Some(validator) = inactive_validator_map.get_mut(&session_key_pair.0) {
                    validator.active_next_session = validator.next_session_keys == session_keys;
                }
            }
            debug!(
                "Got {} queued session keys for the next session.",
                session_key_pairs.len()
            );
        }
        // get reward destinations
        {
            debug!("Get reward destinations.");
            let keys: Vec<String> = inactive_validator_map
                .values()
                .map(|validator| {
                    get_storage_map_key(&self.metadata, "Staking", "Payee", &validator.account.id)
                })
                .collect();

            for chunk in keys.chunks(KEY_QUERY_PAGE_SIZE) {
                let chunk_values: Vec<StorageChangeSet<String>> = self
                    .ws_client
                    .request(
                        "state_queryStorageAt",
                        JsonRpcParams::Array(vec![chunk.into(), block_hash.into()]),
                    )
                    .await?;

                for (storage_key, data) in chunk_values[0].changes.iter() {
                    if let Some(data) = data {
                        let account_id = self.account_id_from_storage_key(storage_key);
                        let bytes: &[u8] = &data.0;
                        let reward_destination = RewardDestination::from_bytes(bytes).unwrap();
                        let validator = inactive_validator_map.get_mut(&account_id).unwrap();
                        validator.reward_destination = reward_destination;
                    }
                }
            }
            debug!("Got reward destinations.");
        }
        // get slashings
        {
            debug!("Get slashings.");
            let keys: Vec<String> = inactive_validator_map
                .values()
                .map(|validator| {
                    get_storage_map_key(
                        &self.metadata,
                        "Staking",
                        "ValidatorSlashInEra",
                        &validator.account.id,
                    )
                })
                .collect();
            let mut number_of_slashed_validators = 0;
            for chunk in keys.chunks(KEY_QUERY_PAGE_SIZE) {
                let chunk_values: Vec<StorageChangeSet<String>> = self
                    .ws_client
                    .request(
                        "state_queryStorageAt",
                        JsonRpcParams::Array(vec![chunk.into(), block_hash.into()]),
                    )
                    .await?;

                for (storage_key, data) in chunk_values[0].changes.iter() {
                    if data.is_some() {
                        let account_id = self.account_id_from_storage_key(storage_key);
                        if let Some(validator) = inactive_validator_map.get_mut(&account_id) {
                            number_of_slashed_validators += 1;
                            validator.slashed = true;
                        }
                    }
                }
            }
            debug!("Got {} slashings.", number_of_slashed_validators);
        }

        // get nominations
        {
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

            debug!("{} nominations.", all_keys.len());
            let mut nomination_map: HashMap<AccountId, Nomination> = HashMap::new();
            for chunk in all_keys.chunks(KEY_QUERY_PAGE_SIZE) {
                let chunk_values: Vec<StorageChangeSet<String>> = self
                    .ws_client
                    .request(
                        "state_queryStorageAt",
                        JsonRpcParams::Array(vec![chunk.into(), block_hash.into()]),
                    )
                    .await?;
                for (storage_key, data) in chunk_values[0].changes.iter() {
                    if let Some(data) = data {
                        let account_id = self.account_id_from_storage_key(storage_key);
                        let bytes: &[u8] = &data.0;
                        let nomination = Nomination::from_bytes(bytes, account_id).unwrap();
                        nomination_map.insert(nomination.nominator_account.id.clone(), nomination);
                    }
                }
            }
            debug!(
                "Got {} nominations. Get nominator account details...",
                nomination_map.len()
            );
            // get nominator account details
            {
                let nominator_account_ids: Vec<AccountId> =
                    nomination_map.keys().cloned().collect();
                for account_id_chunk in nominator_account_ids.chunks(KEY_QUERY_PAGE_SIZE) {
                    let accounts = self.get_accounts(account_id_chunk, block_hash).await?;
                    for account in accounts {
                        nomination_map
                            .get_mut(&account.id)
                            .unwrap()
                            .nominator_account = account.clone();
                    }
                }
                debug!("Completed fetching nominator account details.");
            }
            let mut controller_storage_keys: Vec<String> = nomination_map
                .keys()
                .map(|nominator_account_id| {
                    get_storage_map_key(&self.metadata, "Staking", "Bonded", &nominator_account_id)
                })
                .collect();
            // add validator addresses
            for validator_account_id in inactive_validator_map.keys() {
                controller_storage_keys.push(get_storage_map_key(
                    &self.metadata,
                    "Staking",
                    "Bonded",
                    validator_account_id,
                ))
            }
            let mut controller_account_id_map: HashMap<AccountId, AccountId> = HashMap::new();
            let mut validator_controller_account_ids: Vec<AccountId> = Vec::new();
            for chunk in controller_storage_keys.chunks(KEY_QUERY_PAGE_SIZE) {
                let chunk_values: Vec<StorageChangeSet<String>> = self
                    .ws_client
                    .request(
                        "state_queryStorageAt",
                        JsonRpcParams::Array(vec![chunk.into(), block_hash.into()]),
                    )
                    .await?;
                for (storage_key, data) in chunk_values[0].changes.iter() {
                    if let Some(data) = data {
                        let account_id = self.account_id_from_storage_key(storage_key);
                        let mut bytes: &[u8] = &data.0;
                        let controller_account_id: AccountId = Decode::decode(&mut bytes).unwrap();
                        if let Some(nomination) = nomination_map.get_mut(&account_id) {
                            nomination.controller_account_id = controller_account_id.clone();
                        } else {
                            validator_controller_account_ids.push(controller_account_id.clone())
                        }
                        controller_account_id_map.insert(account_id, controller_account_id);
                    }
                }
            }
            let controller_account_ids: Vec<AccountId> =
                controller_account_id_map.values().cloned().collect();
            debug!(
                "Got {} controller account ids.",
                controller_account_ids.len()
            );
            // get validator controller account details
            {
                let mut validator_controller_account_map: HashMap<AccountId, Account> =
                    HashMap::new();
                for controller_account_id_chunk in
                    validator_controller_account_ids.chunks(KEY_QUERY_PAGE_SIZE)
                {
                    let accounts = self
                        .get_accounts(controller_account_id_chunk, block_hash)
                        .await?;
                    for account in accounts {
                        validator_controller_account_map.insert(account.id.clone(), account);
                    }
                }
                debug!(
                    "Got {} validator controller account details.",
                    validator_controller_account_map.len()
                );
                for inactive_validator in inactive_validator_map.values_mut() {
                    let controller_account = validator_controller_account_map
                        .get(
                            controller_account_id_map
                                .get(&inactive_validator.account.id)
                                .unwrap(),
                        )
                        .unwrap();
                    inactive_validator.controller_account = controller_account.clone();
                }
                debug!("Completed fetching validator controller account details.");
            }
            debug!("Get nominations and self stakes.");
            // her biri için bonding'i al (staking.bonded)
            let ledger_storage_keys: Vec<String> = controller_account_ids
                .iter()
                .map(|controller_account_id| {
                    get_storage_map_key(&self.metadata, "Staking", "Ledger", &controller_account_id)
                })
                .collect();
            // her biri için bonded miktarı al (staking.ledger)
            for chunk in ledger_storage_keys.chunks(KEY_QUERY_PAGE_SIZE) {
                let chunk_values: Vec<StorageChangeSet<String>> = self
                    .ws_client
                    .request(
                        "state_queryStorageAt",
                        JsonRpcParams::Array(vec![chunk.into(), block_hash.into()]),
                    )
                    .await?;
                for (_, data) in chunk_values[0].changes.iter() {
                    if let Some(data) = data {
                        let bytes: &[u8] = &data.0;
                        let stake: Stake = Stake::from_bytes(bytes).unwrap();
                        let account_id = &stake.stash_account_id;
                        if let Some(nomination) = nomination_map.get_mut(account_id) {
                            nomination.stake = stake;
                        } else {
                            let validator = inactive_validator_map.get_mut(account_id).unwrap();
                            validator.self_stake = stake;
                        }
                    }
                }
            }
            debug!("Got all stakes.");
            for nomination in nomination_map.values() {
                for account_id in nomination.target_account_ids.iter() {
                    if let Some(validator) = inactive_validator_map.get_mut(account_id) {
                        validator.nominations.push(nomination.clone());
                        validator.oversubscribed = validator.nominations.len()
                            > max_nominator_rewarded_per_validator as usize;
                    }
                }
            }
            for validator in inactive_validator_map.values_mut() {
                validator.nominations.sort_by_key(|nomination| {
                    let mut hasher = DefaultHasher::new();
                    nomination.nominator_account.id.hash(&mut hasher);
                    hasher.finish()
                });
            }
        }

        // get validator prefs
        {
            debug!("start :: get validator prefs all");
            let values: Vec<StorageChangeSet<String>> = self
                .ws_client
                .request(
                    "state_queryStorageAt",
                    JsonRpcParams::Array(vec![all_keys.into(), block_hash.into()]),
                )
                .await?;
            for (storage_key, data) in values[0].changes.iter() {
                if let Some(data) = data {
                    let bytes: &[u8] = &data.0;
                    let preferences = ValidatorPreferences::from_bytes(bytes).unwrap();
                    let validator_account_id = self.account_id_from_storage_key(storage_key);
                    let validator = inactive_validator_map
                        .get_mut(&validator_account_id)
                        .unwrap();
                    validator.preferences = preferences;
                }
            }
            debug!("Got all validator prefs.");
        }
        debug!("It's done baby!");
        Ok(inactive_validator_map
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
    pub async fn get_era_total_validator_reward(&self, era_index: u32) -> anyhow::Result<Balance> {
        let params = get_rpc_storage_map_params(
            &self.metadata,
            "Staking",
            "ErasValidatorReward",
            &era_index,
            None,
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
                .request(
                    "state_queryStorageAt",
                    JsonRpcParams::Array(vec![chunk.into(), block_hash.into()]),
                )
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
        Ok(decode_hex_string(hex_string.as_str())?)
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

    pub async fn get_block_events(&self, block_hash: &str) -> anyhow::Result<Vec<SubstrateEvent>> {
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
        SubstrateEvent::decode_events(&self.chain, &self.metadata, &mut event_bytes)
    }

    pub async fn get_block_extrinsics(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<SubstrateExtrinsic>> {
        let block = self.get_block(block_hash).await?;
        Ok(SubstrateExtrinsic::decode_extrinsics(
            &self.chain,
            &self.metadata,
            block,
        )?)
    }

    /// Get the number of all validation intents at the given block.
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
        Ok(LastRuntimeUpgradeInfo::from_substrate_hex_string(
            hex_string,
        )?)
    }

    async fn subscribe_to_blocks<F>(
        &self,
        subscribe_method_name: &str,
        unsubscribe_method_name: &str,
        callback: F,
    ) -> anyhow::Result<()>
    where
        F: Fn(BlockHeader),
    {
        let mut subscription: Subscription<BlockHeader> = self
            .ws_client
            .subscribe(
                subscribe_method_name,
                JsonRpcParams::NoParams,
                unsubscribe_method_name,
            )
            .await?;
        loop {
            let block_header = subscription.next().await?;
            match block_header {
                Some(header) => callback(header),
                None => {
                    error!("Empty block header. Will exit new block subscription.");
                    break;
                }
            }
        }
        Ok(())
    }

    /// Subscribes to new blocks.
    pub async fn subscribe_to_new_blocks<F>(&self, callback: F) -> anyhow::Result<()>
    where
        F: Fn(BlockHeader),
    {
        self.subscribe_to_blocks(
            "chain_subscribeNewHeads",
            "chain_unsubscribeNewHeads",
            callback,
        )
        .await
    }

    /// Subscribes to finalized blocks.
    pub async fn subscribe_to_finalized_blocks<F>(&self, callback: F) -> anyhow::Result<()>
    where
        F: Fn(BlockHeader),
    {
        self.subscribe_to_blocks(
            "chain_subscribeFinalizedHeads",
            "chain_unsubscribeFinalizedHeads",
            callback,
        )
        .await
    }
}
