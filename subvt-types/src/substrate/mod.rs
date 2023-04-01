//! Substrate-related types.
//! Mostly translations of the native Substrate runtime types.

use crate::crypto::AccountId;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use frame_support::traits::ConstU32;
pub use pallet_democracy::Conviction as DemocracyConviction;
pub use pallet_democracy::Voting as DemocracyVoting;
use pallet_identity::{Data, Judgement, Registration};
use pallet_staking::{Exposure, UnlockChunk, ValidatorPrefs};
use parity_scale_codec::{Decode, Encode, Error, Input};
pub use polkadot_primitives::v2::{ScrapedOnChainVotes, ValidityAttestation};
use serde::{Deserialize, Serialize};
use sp_consensus_babe::digests::PreDigest;
use sp_core::bounded::BoundedVec;
use sp_core::crypto::{AccountId32, Ss58AddressFormat};
use sp_runtime::DigestItem;
use sp_staking::EraIndex;
use std::collections::BTreeMap;
use std::convert::From;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use subvt_utility::decode_hex_string;
use subvt_utility::text::get_condensed_address;

pub type CallHash = [u8; 32];
pub type OpaqueTimeSlot = Vec<u8>;
pub type Balance = polkadot_core_primitives::Balance;

pub mod bits;
pub mod democracy;
pub mod error;
#[macro_use]
pub mod event;
pub mod extrinsic;
pub mod legacy;
pub mod metadata;
pub mod para;

pub type BlockNumber = polkadot_core_primitives::BlockNumber;

#[derive(Default)]
pub struct LastRuntimeUpgradeInfo {
    pub spec_version: u32,
    pub spec_name: String,
}

impl From<frame_system::LastRuntimeUpgradeInfo> for LastRuntimeUpgradeInfo {
    fn from(upgrade: frame_system::LastRuntimeUpgradeInfo) -> Self {
        Self {
            spec_version: upgrade.spec_version.0,
            spec_name: upgrade.spec_name.to_string(),
        }
    }
}

impl LastRuntimeUpgradeInfo {
    pub fn from_substrate_hex_string(hex_string: String) -> anyhow::Result<Self> {
        Ok(decode_hex_string::<frame_system::LastRuntimeUpgradeInfo>(&hex_string)?.into())
    }
}

/// Chain type.
pub enum Chain {
    Kusama,
    Polkadot,
    Westend,
}

impl FromStr for Chain {
    type Err = std::string::ParseError;

    /// Get chain from string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kusama" | "ksm" => Ok(Self::Kusama),
            "polkadot" | "dot" => Ok(Self::Polkadot),
            "westend" | "wnd" => Ok(Self::Westend),
            _ => panic!("Unkown chain: {s}"),
        }
    }
}

impl Chain {
    /// SS58 encoding format for the chain.
    fn get_ss58_address_format(&self) -> Ss58AddressFormat {
        match self {
            Self::Kusama => Ss58AddressFormat::from(2u16),
            Self::Polkadot => Ss58AddressFormat::from(0u16),
            Self::Westend => Ss58AddressFormat::from(42u16),
        }
    }

    pub fn sp_core_set_default_ss58_version(&self) {
        sp_core::crypto::set_default_ss58_version(self.get_ss58_address_format())
    }
}

/// System properties as fetched from the node RPC interface.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemProperties {
    pub ss_58_format: u8,
    pub token_decimals: u32,
    pub token_symbol: String,
}

#[derive(Debug, Decode, Clone, Eq, PartialEq)]
pub enum MultiAddress {
    Id(AccountId),
    Index(#[codec(compact)] u32),
    Raw(Vec<u8>),
    Address32([u8; 32]),
    Address20([u8; 20]),
}

impl MultiAddress {
    pub fn get_account_id(&self) -> Option<AccountId> {
        match self {
            MultiAddress::Id(account_id) => Some(*account_id),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Account {
    pub id: AccountId,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityRegistration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Box<Option<Account>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub killed_at: Option<u64>,
}

impl Account {
    pub fn get_confirmed(&self) -> bool {
        let self_confirmed = if let Some(identity) = &self.identity {
            identity.confirmed
        } else {
            false
        };
        let parent_confirmed = if let Some(parent_account) = &*self.parent {
            parent_account.get_confirmed()
        } else {
            false
        };
        self_confirmed || parent_confirmed
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = if let Some(parent) = &*self.parent {
            if let Some(child_display) = &self.child_display {
                format!("{parent} / {child_display}")
            } else {
                self.address.clone()
            }
        } else if let Some(identity) = &self.identity {
            if let Some(display) = &identity.display {
                display.clone()
            } else {
                self.address.clone()
            }
        } else {
            self.address.clone()
        };
        write!(f, "{display}")
    }
}

impl Account {
    pub fn get_display(&self) -> Option<String> {
        if let Some(identity) = &self.identity {
            identity.display.clone()
        } else {
            None
        }
    }

    pub fn get_parent_display(&self) -> Option<String> {
        if let Some(parent) = &*self.parent {
            if let Some(identity) = &parent.identity {
                identity.display.clone()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_full_display(&self) -> Option<String> {
        if let Some(identity) = &self.identity {
            if let Some(display) = &identity.display {
                return Some(display.clone());
            }
        } else if let Some(parent) = &*self.parent {
            if let Some(identity) = &parent.identity {
                if let Some(parent_display) = &identity.display {
                    if let Some(child_display) = &self.child_display {
                        return Some(format!("{parent_display} / {child_display}"));
                    }
                }
            }
        }
        None
    }

    pub fn get_display_or_condensed_address(&self, address_side_limit: Option<usize>) -> String {
        if let Some(display) = self.get_full_display() {
            display
        } else {
            get_condensed_address(&self.address, address_side_limit)
        }
    }
}

/// Block wrapper as returned by the RPC method.
#[derive(Serialize, Deserialize, Debug)]
pub struct BlockWrapper {
    pub block: Block,
}

/// Inner block response.
#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub extrinsics: Vec<String>,
}

/// A block's header as fetched from the node RPC interface.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub digest: EventDigest,
    pub extrinsics_root: String,
    pub number: String,
    pub parent_hash: String,
    pub state_root: String,
}

impl BlockHeader {
    /// Number from the hex string.
    pub fn get_number(&self) -> anyhow::Result<u64> {
        let number = u64::from_str_radix(self.number.trim_start_matches("0x"), 16)?;
        Ok(number)
    }

    fn authority_index_from_log_bytes(consensus_engine: &str, mut bytes: &[u8]) -> Option<usize> {
        match consensus_engine {
            "BABE" => {
                let digest: PreDigest = Decode::decode(&mut bytes).unwrap();
                let authority_index = match digest {
                    PreDigest::Primary(digest) => digest.authority_index,
                    PreDigest::SecondaryPlain(digest) => digest.authority_index,
                    PreDigest::SecondaryVRF(digest) => digest.authority_index,
                };
                Some(authority_index as usize)
            }
            "aura" => {
                log::error!("Consensus engine [{}] not supported.", consensus_engine);
                None
            }
            "FRNK" => {
                // GRANDPA
                log::error!("Consensus engine [{}] not supported.", consensus_engine);
                None
            }
            "pow_" => {
                log::error!("Consensus engine [{}] not supported.", consensus_engine);
                None
            }
            _ => {
                log::error!("Unknown consensus engine [{}].", consensus_engine);
                None
            }
        }
    }

    pub fn get_validator_index(&self) -> Option<usize> {
        let mut validator_index: Option<usize> = None;
        for log_string in &self.digest.logs {
            let log_hex_string = log_string.trim_start_matches("0x");
            let mut log_bytes: &[u8] = &hex::decode(log_hex_string).unwrap();
            let digest_item: DigestItem = Decode::decode(&mut log_bytes).unwrap();
            match digest_item {
                DigestItem::PreRuntime(consensus_engine_id, bytes) => {
                    let consensus_engine = std::str::from_utf8(&consensus_engine_id).unwrap();
                    validator_index =
                        BlockHeader::authority_index_from_log_bytes(consensus_engine, &bytes);
                }
                DigestItem::Consensus(consensus_engine_id, bytes) => {
                    if validator_index.is_none() {
                        let consensus_engine = std::str::from_utf8(&consensus_engine_id).unwrap();
                        validator_index =
                            BlockHeader::authority_index_from_log_bytes(consensus_engine, &bytes);
                    }
                }
                DigestItem::Seal(consensus_engine_id, bytes) => {
                    if validator_index.is_none() {
                        let consensus_engine = std::str::from_utf8(&consensus_engine_id).unwrap();
                        validator_index =
                            BlockHeader::authority_index_from_log_bytes(consensus_engine, &bytes);
                    }
                }
                DigestItem::RuntimeEnvironmentUpdated => {
                    log::warn!(
                        "Log type: RuntimeEnvironmentUpdated. Cannot get author validator index."
                    );
                }
                DigestItem::Other(_) => {
                    log::warn!("Unknown log type. Cannot get author validator index.");
                }
            }
        }
        if validator_index.is_none() {
            log::error!("Author validator index not found.");
        }
        validator_index
    }
}

/// Part of the block header.
#[derive(Serialize, Deserialize, Debug)]
pub struct EventDigest {
    logs: Vec<String>,
}

/// Active era as represented in the Substrate runtime.
#[derive(Encode, Decode)]
struct SubstrateActiveEraInfo {
    index: u32,
    start_timestamp_millis: Option<u64>,
}

/// Era as represented in the SubVT domain.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Era {
    pub index: u32,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
}

impl Era {
    /// Era from a hex string (e.g. `0x0563ad5e...`).
    pub fn from(hex_string: &str, era_duration_millis: u64) -> anyhow::Result<Era> {
        let active_era_info: SubstrateActiveEraInfo = decode_hex_string(hex_string)?;
        let start_timestamp = active_era_info.start_timestamp_millis.unwrap();
        let end_timestamp = start_timestamp + era_duration_millis;
        Ok(Era {
            index: active_era_info.index,
            start_timestamp,
            end_timestamp,
        })
    }
}

impl Era {
    pub fn get_start_date_time(&self) -> DateTime<Utc> {
        match Utc::timestamp_opt(&Utc, self.start_timestamp as i64 / 1000, 0) {
            LocalResult::Single(date_time) => date_time,
            _ => panic!("Invalid era start date time."),
        }
    }

    pub fn get_end_date_time(&self) -> DateTime<Utc> {
        match Utc::timestamp_opt(&Utc, self.end_timestamp as i64 / 1000, 0) {
            LocalResult::Single(date_time) => date_time,
            _ => panic!("Invalid era end date time."),
        }
    }
}

/// Epoch as represented in the SubVT domain.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Epoch {
    pub index: u64,
    pub start_block_number: u32,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
}

impl Epoch {
    pub fn get_start_date_time(&self) -> DateTime<Utc> {
        match Utc::timestamp_opt(&Utc, self.start_timestamp as i64 / 1000, 0) {
            LocalResult::Single(date_time) => date_time,
            _ => panic!("Invalid epoch start date time."),
        }
    }

    pub fn get_end_date_time(&self) -> DateTime<Utc> {
        match Utc::timestamp_opt(&Utc, self.end_timestamp as i64 / 1000, 0) {
            LocalResult::Single(date_time) => date_time,
            _ => panic!("Invalid epoch start date time."),
        }
    }
}

/// A nominator's active stake on a validator.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct NominatorStake {
    pub account: Account,
    pub stake: Balance,
}

/// Active staking information for a single active validator. Contains the validator account id,
/// self stake, total stake and each nominator's active stake on the validator.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ValidatorStake {
    pub account: Account,
    pub self_stake: Balance,
    pub total_stake: Balance,
    pub nominators: Vec<NominatorStake>,
}

impl ValidatorStake {
    pub fn from_bytes(mut bytes: &[u8], validator_account_id: AccountId) -> anyhow::Result<Self> {
        let exposure: Exposure<AccountId, Balance> = Decode::decode(&mut bytes)?;
        let mut nominators: Vec<NominatorStake> = Vec::new();
        for other in exposure.others {
            let stake = other.value;
            let account = Account {
                id: other.who,
                address: other.who.to_ss58_check(),
                ..Default::default()
            };
            nominators.push(NominatorStake { account, stake });
        }
        let validator_stake = Self {
            account: Account {
                id: validator_account_id,
                address: validator_account_id.to_ss58_check(),
                ..Default::default()
            },
            self_stake: exposure.own,
            total_stake: exposure.total,
            nominators,
        };
        Ok(validator_stake)
    }
}

/// A collection of all active stakers in an era. See `ValidatorStake` too for details.
pub struct EraStakers {
    pub era: Era,
    pub stakers: Vec<ValidatorStake>,
}

impl EraStakers {
    /// Gets the total stake in era.
    pub fn total_stake(&self) -> Balance {
        self.stakers
            .iter()
            .map(|validator_stake| validator_stake.total_stake)
            .sum()
    }

    /// Gets the minimum stake backing an active validator. Returns validator account id and stake.
    pub fn min_stake(&self) -> (Account, Balance) {
        let validator_stake = self
            .stakers
            .iter()
            .min_by_key(|validator_stake| validator_stake.total_stake)
            .unwrap();
        (validator_stake.account.clone(), validator_stake.total_stake)
    }

    /// Gets the maximum stake backing an active validator. Returns validator account id and stake.
    pub fn max_stake(&self) -> (Account, Balance) {
        let validator_stake = self
            .stakers
            .iter()
            .max_by_key(|validator_stake| validator_stake.total_stake)
            .unwrap();
        (validator_stake.account.clone(), validator_stake.total_stake)
    }

    /// Gets the average of all stakes backing all active validators.
    pub fn average_stake(&self) -> Balance {
        let sum = self
            .stakers
            .iter()
            .map(|validator_stake| validator_stake.total_stake)
            .sum::<Balance>();
        sum / self.stakers.len() as Balance
    }

    /// Gets the median of all stakes backing all active validators.
    pub fn median_stake(&self) -> Balance {
        let mid = self.stakers.len() / 2;
        self.stakers[mid].total_stake
    }
}

/// Total reward points earned over an era. It will contain the points earned so far
/// for an active era.
#[derive(Encode, Decode, Serialize)]
pub struct EraRewardPoints {
    pub total: u32,
    pub individual: BTreeMap<AccountId32, u32>,
}

/// Validator commission and block preferences.
#[derive(Clone, Debug, Encode, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ValidatorPreferences {
    pub commission_per_billion: u32,
    pub blocks_nominations: bool,
}

impl Decode for ValidatorPreferences {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let preferences: ValidatorPrefs = Decode::decode(input)?;
        Ok(ValidatorPreferences {
            commission_per_billion: preferences.commission.deconstruct(),
            blocks_nominations: preferences.blocked,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct IdentityRegistration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub riot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<String>,
    pub confirmed: bool,
}

pub fn data_to_string(data: Data) -> Option<String> {
    match data {
        Data::Raw(raw) => {
            let maybe_string = String::from_utf8(raw.into_inner());
            if let Ok(string) = maybe_string {
                Some(string)
            } else {
                None
            }
        }
        _ => None,
    }
}

impl IdentityRegistration {
    pub fn from_bytes(mut bytes: &[u8]) -> anyhow::Result<Self> {
        let registration: Registration<Balance, ConstU32<{ u32::MAX }>, ConstU32<{ u32::MAX }>> =
            Decode::decode(&mut bytes)?;
        let display = data_to_string(registration.info.display);
        let email = data_to_string(registration.info.email);
        let riot = data_to_string(registration.info.riot);
        let twitter = data_to_string(registration.info.twitter);
        let web = data_to_string(registration.info.web);
        let mut confirmed = true;
        for judgement in registration.judgements {
            confirmed &= match judgement.1 {
                Judgement::Reasonable | Judgement::KnownGood => true,
                Judgement::Unknown => false,
                Judgement::FeePaid(_) => false,
                Judgement::OutOfDate => false,
                Judgement::LowQuality => false,
                Judgement::Erroneous => false,
            };
        }
        Ok(IdentityRegistration {
            display,
            email,
            riot,
            twitter,
            web,
            confirmed,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct IdentityRegistrationSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    pub confirmed: bool,
}

impl From<&IdentityRegistration> for IdentityRegistrationSummary {
    fn from(identity: &IdentityRegistration) -> IdentityRegistrationSummary {
        IdentityRegistrationSummary {
            display: identity.display.clone(),
            confirmed: identity.confirmed,
        }
    }
}

pub type SuperAccountId = (AccountId, Data);

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct NominationSummary {
    pub stash_account: Account,
    pub submission_era_index: u32,
    pub nominee_count: u16,
    pub stake: Stake,
}

impl From<&Nomination> for NominationSummary {
    fn from(nomination: &Nomination) -> NominationSummary {
        NominationSummary {
            stash_account: nomination.stash_account.clone(),
            submission_era_index: nomination.submission_era_index,
            nominee_count: nomination.target_account_ids.len() as u16,
            stake: nomination.stake.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Nomination {
    pub stash_account: Account,
    pub submission_era_index: u32,
    pub target_account_ids: Vec<AccountId>,
    pub stake: Stake,
}

impl Nomination {
    pub fn from_bytes(mut bytes: &[u8], account_id: AccountId) -> anyhow::Result<Self> {
        let nomination: (Vec<AccountId>, EraIndex, bool) = Decode::decode(&mut bytes)?;
        Ok(Nomination {
            stash_account: Account {
                id: account_id,
                address: account_id.to_ss58_check(),
                ..Default::default()
            },
            submission_era_index: nomination.1,
            target_account_ids: nomination.0,
            ..Default::default()
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct InactiveNominationsSummary {
    pub nomination_count: u16,
    pub total_amount: Balance,
}

impl From<&Vec<NominationSummary>> for InactiveNominationsSummary {
    fn from(nominations: &Vec<NominationSummary>) -> InactiveNominationsSummary {
        InactiveNominationsSummary {
            nomination_count: nominations.len() as u16,
            total_amount: nominations.iter().fold(0, |a, b| a + b.stake.active_amount),
        }
    }
}

#[derive(Decode)]
struct StakingLedger {
    pub stash: AccountId,
    #[codec(compact)]
    pub total: Balance,
    #[codec(compact)]
    pub active: Balance,
    pub _unlocking: BoundedVec<UnlockChunk<Balance>, ConstU32<{ u32::MAX }>>,
    pub _claimed_rewards: BoundedVec<EraIndex, ConstU32<{ u32::MAX }>>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Stake {
    pub stash_account_id: AccountId,
    pub total_amount: Balance,
    pub active_amount: Balance,
    // pub claimed_era_indices: Vec<u32>,
}

impl Stake {
    pub fn from_bytes(mut bytes: &[u8]) -> anyhow::Result<Self> {
        let ledger: StakingLedger = Decode::decode(&mut bytes)?;
        let stake = Self {
            stash_account_id: ledger.stash,
            total_amount: ledger.total,
            active_amount: ledger.active,
            // claimed_era_indices: ledger.claimed_rewards,
        };
        Ok(stake)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StakeSummary {
    pub stash_account_id: AccountId,
    pub active_amount: Balance,
}

impl From<&Stake> for StakeSummary {
    fn from(stake: &Stake) -> StakeSummary {
        StakeSummary {
            stash_account_id: stake.stash_account_id,
            active_amount: stake.active_amount,
        }
    }
}

#[derive(Clone, Decode, Debug, Deserialize, Encode, Eq, Hash, PartialEq, Serialize)]
#[serde(tag = "destination_type", content = "destination")]
pub enum RewardDestination {
    Staked,
    Stash,
    Controller,
    Account(AccountId),
    None,
}

impl Default for RewardDestination {
    fn default() -> Self {
        Self::None
    }
}

impl RewardDestination {
    pub fn from_bytes(mut bytes: &[u8]) -> anyhow::Result<Self> {
        let destination: pallet_staking::RewardDestination<AccountId> = Decode::decode(&mut bytes)?;
        let destination = match destination {
            pallet_staking::RewardDestination::Staked => Self::Staked,
            pallet_staking::RewardDestination::Stash => Self::Stash,
            pallet_staking::RewardDestination::Controller => Self::Controller,
            pallet_staking::RewardDestination::Account(account_id) => Self::Account(account_id),
            pallet_staking::RewardDestination::None => Self::None,
        };
        Ok(destination)
    }
}

#[derive(Clone, Debug, Decode)]
pub enum ProxyType {
    Any,
    NonTransfer,
    Governance,
    Staking,
    IdentityJudgement,
    CancelProxy,
    Auction,
    Society,
}
