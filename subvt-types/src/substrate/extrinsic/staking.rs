use crate::crypto::AccountId;
use crate::substrate::error::DecodeError;
use crate::substrate::extrinsic::{Signature, SubstrateExtrinsic};
use crate::substrate::{Balance, MultiAddress, RewardDestination, ValidatorPreferences};
use parity_scale_codec::{Compact, Decode};
use sp_staking::EraIndex;

const BOND: &str = "bond";
const NOMINATE: &str = "nominate";
const PAYOUT_STAKERS: &str = "payout_stakers";
const PAYOUT_STAKERS_BY_PAGE: &str = "payout_stakers_by_page";
const VALIDATE: &str = "validate";

#[derive(Clone, Debug)]
pub enum StakingExtrinsic {
    Bond {
        maybe_signature: Option<Signature>,
        amount: Balance,
        reward_destination: RewardDestination,
    },
    Nominate {
        maybe_signature: Option<Signature>,
        targets: Vec<MultiAddress>,
    },
    PayoutStakers {
        maybe_signature: Option<Signature>,
        validator_account_id: AccountId,
        era_index: EraIndex,
    },
    PayoutStakersByPage {
        maybe_signature: Option<Signature>,
        validator_account_id: AccountId,
        era_index: EraIndex,
        page_index: u32,
    },
    Validate {
        maybe_signature: Option<Signature>,
        preferences: ValidatorPreferences,
    },
}

impl StakingExtrinsic {
    pub fn decode(
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            BOND => {
                let compact_amount: Compact<Balance> = Decode::decode(bytes)?;
                let reward_destination: RewardDestination = Decode::decode(bytes)?;
                Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Bond {
                    maybe_signature: maybe_signature.clone(),
                    amount: compact_amount.0,
                    reward_destination,
                }))
            }
            NOMINATE => Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Nominate {
                maybe_signature: maybe_signature.clone(),
                targets: Decode::decode(bytes)?,
            })),
            PAYOUT_STAKERS => Some(SubstrateExtrinsic::Staking(
                StakingExtrinsic::PayoutStakers {
                    maybe_signature: maybe_signature.clone(),
                    validator_account_id: Decode::decode(bytes)?,
                    era_index: Decode::decode(bytes)?,
                },
            )),
            PAYOUT_STAKERS_BY_PAGE => Some(SubstrateExtrinsic::Staking(
                StakingExtrinsic::PayoutStakersByPage {
                    maybe_signature: maybe_signature.clone(),
                    validator_account_id: Decode::decode(bytes)?,
                    era_index: Decode::decode(bytes)?,
                    page_index: Decode::decode(bytes)?,
                },
            )),
            VALIDATE => Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Validate {
                maybe_signature: maybe_signature.clone(),
                preferences: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}
