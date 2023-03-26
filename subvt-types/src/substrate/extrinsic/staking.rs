use crate::crypto::AccountId;
use crate::substrate::error::DecodeError;
use crate::substrate::extrinsic::{Signature, SubstrateExtrinsic};
use crate::substrate::{Balance, MultiAddress, RewardDestination, ValidatorPreferences};
use parity_scale_codec::{Compact, Decode};
use sp_staking::EraIndex;

const BOND: &str = "bond";
const NOMINATE: &str = "nominate";
const PAYOUT_STAKERS: &str = "payout_stakers";
const SET_CONTROLLER: &str = "set_controller";
const VALIDATE: &str = "validate";

#[derive(Clone, Debug)]
pub enum StakingExtrinsic {
    Bond {
        maybe_signature: Option<Signature>,
        controller: MultiAddress,
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
    SetController {
        maybe_signature: Option<Signature>,
        controller: MultiAddress,
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
                let controller: MultiAddress = Decode::decode(bytes)?;
                let compact_amount: Compact<Balance> = Decode::decode(bytes)?;
                let reward_destination: RewardDestination = Decode::decode(bytes)?;
                Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Bond {
                    maybe_signature: maybe_signature.clone(),
                    controller,
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
            SET_CONTROLLER => Some(SubstrateExtrinsic::Staking(
                StakingExtrinsic::SetController {
                    maybe_signature: maybe_signature.clone(),
                    controller: Decode::decode(bytes)?,
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
