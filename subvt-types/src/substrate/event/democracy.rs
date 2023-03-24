use crate::crypto::AccountId;
use crate::substrate::error::DecodeError;
use crate::substrate::event::SubstrateEvent;
use crate::substrate::Balance;
use pallet_democracy::{AccountVote, PropIndex, ReferendumIndex, VoteThreshold};
use parity_scale_codec::Decode;

const CANCELLED: &str = "Cancelled";
const DELEGATED: &str = "Delegated";
const NOT_PASSED: &str = "NotPassed";
const PASSED: &str = "Passed";
const PROPOSED: &str = "Proposed";
const SECONDED: &str = "Seconded";
const STARTED: &str = "Started";
const UNDELEGATED: &str = "Undelegated";
const VOTED: &str = "Voted";

#[derive(Clone, Debug)]
pub enum DemocracyEvent {
    Cancelled {
        extrinsic_index: Option<u32>,
        referendum_index: ReferendumIndex,
    },
    Delegated {
        extrinsic_index: Option<u32>,
        original_account_id: AccountId,
        delegate_account_id: AccountId,
    },
    NotPassed {
        extrinsic_index: Option<u32>,
        referendum_index: ReferendumIndex,
    },
    Passed {
        extrinsic_index: Option<u32>,
        referendum_index: ReferendumIndex,
    },
    Proposed {
        extrinsic_index: Option<u32>,
        proposal_index: PropIndex,
        deposit: Balance,
    },
    Seconded {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        proposal_index: PropIndex,
    },
    Started {
        extrinsic_index: Option<u32>,
        referendum_index: ReferendumIndex,
        vote_threshold: VoteThreshold,
    },
    Undelegated {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
    },
    Voted {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        referendum_index: ReferendumIndex,
        vote: AccountVote<Balance>,
    },
}

impl DemocracyEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::Cancelled {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Delegated {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::NotPassed {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Passed {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Proposed {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Seconded {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Started {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Undelegated {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Voted {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl DemocracyEvent {
    pub fn decode(
        _runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            CANCELLED => Some(SubstrateEvent::Democracy(DemocracyEvent::Cancelled {
                extrinsic_index,
                referendum_index: Decode::decode(bytes)?,
            })),
            DELEGATED => Some(SubstrateEvent::Democracy(DemocracyEvent::Delegated {
                extrinsic_index,
                original_account_id: Decode::decode(bytes)?,
                delegate_account_id: Decode::decode(bytes)?,
            })),
            NOT_PASSED => Some(SubstrateEvent::Democracy(DemocracyEvent::NotPassed {
                extrinsic_index,
                referendum_index: Decode::decode(bytes)?,
            })),
            PASSED => Some(SubstrateEvent::Democracy(DemocracyEvent::Passed {
                extrinsic_index,
                referendum_index: Decode::decode(bytes)?,
            })),
            PROPOSED => Some(SubstrateEvent::Democracy(DemocracyEvent::Proposed {
                extrinsic_index,
                proposal_index: Decode::decode(bytes)?,
                deposit: Decode::decode(bytes)?,
            })),
            SECONDED => Some(SubstrateEvent::Democracy(DemocracyEvent::Seconded {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
                proposal_index: Decode::decode(bytes)?,
            })),
            STARTED => Some(SubstrateEvent::Democracy(DemocracyEvent::Started {
                extrinsic_index,
                referendum_index: Decode::decode(bytes)?,
                vote_threshold: Decode::decode(bytes)?,
            })),
            UNDELEGATED => Some(SubstrateEvent::Democracy(DemocracyEvent::Undelegated {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
            })),
            VOTED => Some(SubstrateEvent::Democracy(DemocracyEvent::Voted {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
                referendum_index: Decode::decode(bytes)?,
                vote: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}
