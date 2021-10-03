//! Substrate-related types.
//! Mostly translations of the native Substrate runtime types.

use chrono::{DateTime, Utc, TimeZone};
use crate::crypto::AccountId;
use frame_support::traits::ConstU32;
use pallet_identity::{Data, Judgement, Registration};
use pallet_staking::{Exposure, Nominations, StakingLedger, ValidatorPrefs};
use parity_scale_codec::{Encode, Decode};
use serde::{Deserialize, Serialize};
use sp_core::crypto::{AccountId32, Ss58AddressFormat};
use std::str::FromStr;
use std::collections::BTreeMap;
use std::convert::From;
use std::fmt::{Display, Formatter};
use subvt_utility::decode_hex_string;

pub type CallHash = [u8; 32];
pub type OpaqueTimeSlot = Vec<u8>;

/// Chain type.
pub enum Chain {
    Kusama,
    Polkadot,
    Darwinia,
}

impl FromStr for Chain {
    type Err = std::string::ParseError;

    /// Get chain from string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kusama" | "ksm" => Ok(Self::Kusama),
            "polkadot" | "dot" => Ok(Self::Polkadot),
            "darwinia" => Ok(Self::Darwinia),
            _ => panic!("Unkown chain: {}", s)
        }
    }
}

impl Chain {
    /// SS58 encoding format for the chain.
    fn get_ss58_address_format(&self) -> Ss58AddressFormat {
        match self {
            Self::Kusama => Ss58AddressFormat::KusamaAccount,
            Self::Polkadot => Ss58AddressFormat::PolkadotAccount,
            Self::Darwinia => Ss58AddressFormat::DarwiniaAccount,
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

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Account {
    pub id: AccountId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityRegistration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Box<Option<Account>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_at: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AccountSummary {
    pub id: AccountId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityRegistrationSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Box<Option<AccountSummary>>,
}

impl From<&Account> for AccountSummary {
    fn from(account: &Account) -> AccountSummary {
        AccountSummary {
            id: account.id.clone(),
            identity: account.identity.as_ref().map(|identity| identity.into()),
            parent: if let Some(account) = &*account.parent {
                Box::new(Some(AccountSummary::from(account)))
            } else {
                Box::new(None)
            },
        }
    }
}

/// A block's header as fetched from the node RPC interface.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    digest: EventDigest,
    extrinsics_root: String,
    number: String,
    parent_hash: String,
    state_root: String,
}

impl BlockHeader {
    /// Number from the hex string.
    pub fn get_number(&self) -> anyhow::Result<u64> {
        let number = u64::from_str_radix(
            self.number.trim_start_matches("0x"), 16,
        )?;
        Ok(number)
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
    pub fn from(
        hex_string: &str,
        era_duration_millis: u64,
    ) -> anyhow::Result<Era> {
        let active_era_info: SubstrateActiveEraInfo = decode_hex_string(hex_string)?;
        let start_timestamp_millis = active_era_info.start_timestamp_millis.unwrap();
        let start_timestamp = start_timestamp_millis / 1000;
        let end_timestamp_millis = start_timestamp_millis + era_duration_millis;
        let end_timestamp = end_timestamp_millis / 1000;
        Ok(
            Era {
                index: active_era_info.index,
                start_timestamp,
                end_timestamp,
            }
        )
    }
}

impl Era {
    pub fn get_start_date_time(&self) -> DateTime<Utc> {
        Utc::timestamp(&Utc, self.start_timestamp as i64, 0)
    }

    pub fn get_end_date_time(&self) -> DateTime<Utc> {
        Utc::timestamp(&Utc, self.end_timestamp as i64, 0)
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
        Utc::timestamp(&Utc, self.start_timestamp as i64, 0)
    }

    pub fn get_end_date_time(&self) -> DateTime<Utc> {
        Utc::timestamp(&Utc, self.end_timestamp as i64, 0)
    }
}

/// A nominator's active stake on a validator.
pub struct NominatorStake {
    pub account: Account,
    pub stake: u128,
}

/// Active staking information for a single active validator. Contains the validator account id,
/// self stake, total stake and each nominator's active stake on the validator.
pub struct ValidatorStake {
    pub account: Account,
    pub self_stake: u128,
    pub total_stake: u128,
    pub nominators: Vec<NominatorStake>,
}

impl ValidatorStake {
    pub fn from_bytes(mut bytes: &[u8], validator_account_id: AccountId) -> anyhow::Result<Self> {
        let exposure: Exposure<AccountId, u128> = Decode::decode(&mut bytes)?;
        let mut nominators: Vec<NominatorStake> = Vec::new();
        for other in exposure.others {
            let stake = other.value;
            let account = Account { id: other.who, ..Default::default() };
            nominators.push(NominatorStake { account, stake });
        }
        let validator_stake = Self {
            account: Account {
                id: validator_account_id,
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
    pub fn total_stake(&self) -> u128 {
        self.stakers.iter().map(
            |validator_stake|
                validator_stake.total_stake
        ).sum()
    }

    /// Gets the minimum stake backing an active validator. Returns validator account id and stake.
    pub fn min_stake(&self) -> (Account, u128) {
        let validator_stake = self.stakers.iter().min_by_key(
            |validator_stake|
                validator_stake.total_stake
        ).unwrap();
        (validator_stake.account.clone(), validator_stake.total_stake)
    }

    /// Gets the maximum stake backing an active validator. Returns validator account id and stake.
    pub fn max_stake(&self) -> (Account, u128) {
        let validator_stake = self.stakers.iter().max_by_key(
            |validator_stake|
                validator_stake.total_stake
        ).unwrap();
        (validator_stake.account.clone(), validator_stake.total_stake)
    }

    /// Gets the average of all stakes backing all active validators.
    pub fn average_stake(&self) -> u128 {
        let sum = self.stakers.iter().map(
            |validator_stake| validator_stake.total_stake
        ).sum::<u128>();
        sum / self.stakers.len() as u128
    }

    /// Gets the median of all stakes backing all active validators.
    pub fn median_stake(&self) -> u128 {
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
#[derive(Clone, Debug, Encode, Decode, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ValidatorPreferences {
    pub commission_per_billion: u32,
    pub blocks_nominations: bool,
}

impl ValidatorPreferences {
    pub fn from_bytes(mut bytes: &[u8]) -> anyhow::Result<Self> {
        let preferences: ValidatorPrefs = Decode::decode(&mut bytes)?;
        Ok(
            ValidatorPreferences {
                commission_per_billion: preferences.commission.deconstruct(),
                blocks_nominations: preferences.blocked,
            }
        )
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

impl IdentityRegistration {
    pub fn from_bytes(mut bytes: &[u8]) -> anyhow::Result<Self> {
        fn data_to_string(data: Data) -> Option<String> {
            match data {
                Data::Raw(raw) => {
                    let maybe_string = String::from_utf8(raw.into_inner());
                    if let Ok(string) = maybe_string {
                        Some(string)
                    } else {
                        None
                    }
                }
                _ => None
            }
        }
        let registration: Registration<u128, ConstU32<{ u32::MAX }>, ConstU32<{ u32::MAX }>> = Decode::decode(&mut bytes)?;
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
        Ok(
            IdentityRegistration {
                display,
                email,
                riot,
                twitter,
                web,
                confirmed,
            }
        )
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
pub struct Nomination {
    pub nominator_account: Account,
    pub controller_account_id: AccountId,
    pub submission_era_index: u32,
    pub target_account_ids: Vec<AccountId>,
    pub stake: Stake,
}

impl Nomination {
    pub fn from_bytes(mut bytes: &[u8], account_id: AccountId) -> anyhow::Result<Self> {
        let nomination: Nominations<AccountId> = Decode::decode(&mut bytes)?;
        let submission_era_index: u32 = nomination.submitted_in;
        Ok(
            Nomination {
                nominator_account: Account { id: account_id, ..Default::default() },
                submission_era_index,
                target_account_ids: nomination.targets,
                ..Default::default()
            }
        )
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct NominationsSummary {
    pub nomination_count: u16,
    pub total_active_amount: u128,
}

impl From<&Vec<Nomination>> for NominationsSummary {
    fn from(nominations: &Vec<Nomination>) -> NominationsSummary {
        NominationsSummary {
            nomination_count: nominations.len() as u16,
            total_active_amount: nominations.iter().fold(
                0,
                |a, b| a + b.stake.active_amount,
            ),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Stake {
    pub stash_account_id: AccountId,
    pub total_amount: u128,
    pub active_amount: u128,
    pub claimed_era_indices: Vec<u32>,
}

impl Stake {
    pub fn from_bytes(mut bytes: &[u8]) -> anyhow::Result<Self> {
        let ledger: StakingLedger<AccountId, u128> = Decode::decode(&mut bytes)?;
        let stake = Self {
            stash_account_id: ledger.stash,
            total_amount: ledger.total,
            active_amount: ledger.active,
            claimed_era_indices: ledger.claimed_rewards,
        };
        Ok(stake)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StakeSummary {
    pub stash_account_id: AccountId,
    pub active_amount: u128,
}

impl From<&Stake> for StakeSummary {
    fn from(stake: &Stake) -> StakeSummary {
        StakeSummary {
            stash_account_id: stake.stash_account_id.clone(),
            active_amount: stake.active_amount,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum RewardDestination {
    Staked,
    Stash,
    Controller,
    Account(AccountId),
    None,
}

impl Display for RewardDestination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Staked => f.write_str("Staked"),
            Self::Stash => f.write_str("Stash"),
            Self::Controller => f.write_str("Controller"),
            Self::Account(account_id) => f.write_str(&format!("Account({})", account_id)),
            Self::None => f.write_str("None"),
        }
    }
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