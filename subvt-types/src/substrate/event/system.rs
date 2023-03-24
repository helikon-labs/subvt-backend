use crate::crypto::AccountId;
use crate::substrate::error::DecodeError;
use crate::substrate::event::SubstrateEvent;
use crate::substrate::metadata::{decode_dispatch_error, decode_dispatch_info};
use frame_support::dispatch::{DispatchError, DispatchInfo};
use parity_scale_codec::Decode;

const CODE_UPDATED: &str = "CodeUpdated";
const EXTRINSIC_SUCCESS: &str = "ExtrinsicSuccess";
const EXTRINSIC_FAILED: &str = "ExtrinsicFailed";
const KILLED_ACCOUNT: &str = "KilledAccount";
const NEW_ACCOUNT: &str = "NewAccount";

#[derive(Clone, Debug)]
pub enum SystemEvent {
    CodeUpdated {
        extrinsic_index: Option<u32>,
    },
    ExtrinsicFailed {
        extrinsic_index: Option<u32>,
        dispatch_error: DispatchError,
        dispatch_info: DispatchInfo,
    },
    ExtrinsicSuccess {
        extrinsic_index: Option<u32>,
        dispatch_info: DispatchInfo,
    },
    KilledAccount {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
    },
    NewAccount {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
    },
}

impl SystemEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::CodeUpdated {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::ExtrinsicFailed {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::ExtrinsicSuccess {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::KilledAccount {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::NewAccount {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl SystemEvent {
    pub fn decode(
        runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            CODE_UPDATED => Some(SubstrateEvent::System(SystemEvent::CodeUpdated {
                extrinsic_index,
            })),
            EXTRINSIC_SUCCESS => Some(SubstrateEvent::System(SystemEvent::ExtrinsicSuccess {
                extrinsic_index,
                dispatch_info: decode_dispatch_info(runtime_version, bytes)?,
            })),
            EXTRINSIC_FAILED => Some(SubstrateEvent::System(SystemEvent::ExtrinsicFailed {
                extrinsic_index,
                dispatch_error: decode_dispatch_error(runtime_version, bytes)?,
                dispatch_info: decode_dispatch_info(runtime_version, bytes)?,
            })),
            KILLED_ACCOUNT => Some(SubstrateEvent::System(SystemEvent::KilledAccount {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
            })),
            NEW_ACCOUNT => Some(SubstrateEvent::System(SystemEvent::NewAccount {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}
