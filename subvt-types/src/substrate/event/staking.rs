use crate::crypto::AccountId;
use crate::substrate::error::DecodeError;
use crate::substrate::event::SubstrateEvent;
use crate::substrate::Balance;
use parity_scale_codec::Decode;
use sp_staking::{EraIndex, SessionIndex};

const BONDED: &str = "Bonded";
const CHILLED: &str = "Chilled";
const CHILL: &str = "Chill";
const ERA_PAID: &str = "EraPaid";
const ERA_PAYOUT: &str = "EraPayout";
const KICKED: &str = "Kicked";
const OLD_SLASHING_REPORT_DISCARDED: &str = "OldSlashingReportDiscarded";
const PAYOUT_STARTED: &str = "PayoutStarted";
const REWARDED: &str = "Rewarded";
const REWARD: &str = "Reward";
const SLASHED: &str = "Slashed";
const SLASH: &str = "Slash";
const STAKERS_ELECTED: &str = "StakersElected";
const STAKING_ELECTION: &str = "StakingElection";
const STAKING_ELECTION_FAILED: &str = "StakingElectionFailed";
const UNBONDED: &str = "Unbonded";
const WITHDRAWN: &str = "Withdrawn";

#[derive(Clone, Debug)]
pub enum StakingEvent {
    Bonded {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        balance: Balance,
    },
    Chilled {
        extrinsic_index: Option<u32>,
        stash_account_id: AccountId,
    },
    EraPaid {
        extrinsic_index: Option<u32>,
        era_index: EraIndex,
        validator_payout: Balance,
        remainder: Balance,
    },
    NominatorKicked {
        extrinsic_index: Option<u32>,
        nominator_account_id: AccountId,
        validator_account_id: AccountId,
    },
    OldSlashingReportDiscarded {
        extrinsic_index: Option<u32>,
        session_index: SessionIndex,
    },
    PayoutStarted {
        extrinsic_index: Option<u32>,
        era_index: EraIndex,
        validator_account_id: AccountId,
    },
    Rewarded {
        extrinsic_index: Option<u32>,
        rewardee_account_id: AccountId,
        amount: Balance,
    },
    Slashed {
        extrinsic_index: Option<u32>,
        validator_account_id: AccountId,
        amount: Balance,
    },
    StakersElected {
        extrinsic_index: Option<u32>,
    },
    StakingElectionFailed {
        extrinsic_index: Option<u32>,
    },
    Unbonded {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        amount: Balance,
    },
    Withdrawn {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        amount: Balance,
    },
}

impl StakingEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::Bonded {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Chilled {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::EraPaid {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::NominatorKicked {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::OldSlashingReportDiscarded {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::PayoutStarted {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Rewarded {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Slashed {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::StakersElected {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::StakingElectionFailed {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Unbonded {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Withdrawn {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl StakingEvent {
    pub fn decode(
        _runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            BONDED => Some(SubstrateEvent::Staking(StakingEvent::Bonded {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
                balance: Decode::decode(bytes)?,
            })),
            CHILLED | CHILL => Some(SubstrateEvent::Staking(StakingEvent::Chilled {
                extrinsic_index,
                stash_account_id: Decode::decode(bytes)?,
            })),
            ERA_PAID | ERA_PAYOUT => Some(SubstrateEvent::Staking(StakingEvent::EraPaid {
                extrinsic_index,
                era_index: Decode::decode(bytes)?,
                validator_payout: Decode::decode(bytes)?,
                remainder: Decode::decode(bytes)?,
            })),
            KICKED => Some(SubstrateEvent::Staking(StakingEvent::NominatorKicked {
                extrinsic_index,
                nominator_account_id: Decode::decode(bytes)?,
                validator_account_id: Decode::decode(bytes)?,
            })),
            OLD_SLASHING_REPORT_DISCARDED => Some(SubstrateEvent::Staking(
                StakingEvent::OldSlashingReportDiscarded {
                    extrinsic_index,
                    session_index: Decode::decode(bytes)?,
                },
            )),
            PAYOUT_STARTED => Some(SubstrateEvent::Staking(StakingEvent::PayoutStarted {
                extrinsic_index,
                era_index: Decode::decode(bytes)?,
                validator_account_id: Decode::decode(bytes)?,
            })),
            REWARDED | REWARD => Some(SubstrateEvent::Staking(StakingEvent::Rewarded {
                extrinsic_index,
                rewardee_account_id: Decode::decode(bytes)?,
                amount: Decode::decode(bytes)?,
            })),
            SLASHED | SLASH => Some(SubstrateEvent::Staking(StakingEvent::Slashed {
                extrinsic_index,
                validator_account_id: Decode::decode(bytes)?,
                amount: Decode::decode(bytes)?,
            })),
            STAKERS_ELECTED | STAKING_ELECTION => {
                Some(SubstrateEvent::Staking(StakingEvent::StakersElected {
                    extrinsic_index,
                }))
            }
            STAKING_ELECTION_FAILED => Some(SubstrateEvent::Staking(
                StakingEvent::StakingElectionFailed { extrinsic_index },
            )),
            UNBONDED => Some(SubstrateEvent::Staking(StakingEvent::Unbonded {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
                amount: Decode::decode(bytes)?,
            })),
            WITHDRAWN => Some(SubstrateEvent::Staking(StakingEvent::Withdrawn {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
                amount: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}
