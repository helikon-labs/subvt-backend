use crate::substrate::{error::DecodeError, event::SubstrateEvent, Balance};
use frame_support::traits::Bounded;
use pallet_conviction_voting::Tally;
use pallet_referenda::ReferendumIndex;
use parity_scale_codec::Decode;

const CANCELLED: &str = "Cancelled";
const CONFIRMED: &str = "Confirmed";
const DECISION_STARTED: &str = "DecisionStarted";
const REJECTED: &str = "Rejected";
const SUBMITTED: &str = "Submitted";

#[derive(Clone, Debug)]
pub enum ReferendaEvent {
    Cancelled {
        extrinsic_index: Option<u32>,
        referendum_index: ReferendumIndex,
        tally: Tally<Balance, Balance>,
    },
    Confirmed {
        extrinsic_index: Option<u32>,
        referendum_index: ReferendumIndex,
        tally: Tally<Balance, Balance>,
    },
    DecisionStarted {
        extrinsic_index: Option<u32>,
        referendum_index: ReferendumIndex,
        track_id: u16,
        // type parameter is dummy
        proposal: Bounded<u8>,
        tally: Tally<Balance, Balance>,
    },
    Rejected {
        extrinsic_index: Option<u32>,
        referendum_index: ReferendumIndex,
        tally: Tally<Balance, Balance>,
    },
    Submitted {
        extrinsic_index: Option<u32>,
        referendum_index: ReferendumIndex,
        track_id: u16,
        // type parameter is dummy
        proposal: Bounded<u8>,
    },
}

impl ReferendaEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::Cancelled {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Confirmed {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::DecisionStarted {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Rejected {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::Submitted {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl ReferendaEvent {
    pub fn decode(
        _runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            CANCELLED => Some(SubstrateEvent::Referenda(ReferendaEvent::Cancelled {
                extrinsic_index,
                referendum_index: Decode::decode(bytes)?,
                tally: Decode::decode(bytes)?,
            })),
            CONFIRMED => Some(SubstrateEvent::Referenda(ReferendaEvent::Confirmed {
                extrinsic_index,
                referendum_index: Decode::decode(bytes)?,
                tally: Decode::decode(bytes)?,
            })),
            DECISION_STARTED => Some(SubstrateEvent::Referenda(ReferendaEvent::DecisionStarted {
                extrinsic_index,
                referendum_index: Decode::decode(bytes)?,
                track_id: Decode::decode(bytes)?,
                proposal: Decode::decode(bytes)?,
                tally: Decode::decode(bytes)?,
            })),
            REJECTED => Some(SubstrateEvent::Referenda(ReferendaEvent::Rejected {
                extrinsic_index,
                referendum_index: Decode::decode(bytes)?,
                tally: Decode::decode(bytes)?,
            })),
            SUBMITTED => Some(SubstrateEvent::Referenda(ReferendaEvent::Submitted {
                extrinsic_index,
                referendum_index: Decode::decode(bytes)?,
                track_id: Decode::decode(bytes)?,
                proposal: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}
