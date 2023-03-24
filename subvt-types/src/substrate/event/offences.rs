use crate::substrate::error::DecodeError;
use crate::substrate::event::SubstrateEvent;
use crate::substrate::OpaqueTimeSlot;
use parity_scale_codec::Decode;
use sp_staking::offence::Kind;

const OFFENCE: &str = "Offence";

#[derive(Clone, Debug)]
pub enum OffencesEvent {
    Offence {
        extrinsic_index: Option<u32>,
        offence_kind: Kind,
        time_slot: OpaqueTimeSlot,
    },
}

impl OffencesEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::Offence {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl OffencesEvent {
    pub fn decode(
        _runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            OFFENCE => Some(SubstrateEvent::Offences(OffencesEvent::Offence {
                extrinsic_index,
                offence_kind: Decode::decode(bytes)?,
                time_slot: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}
