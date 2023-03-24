use crate::substrate::error::DecodeError;
use crate::substrate::event::SubstrateEvent;
use crate::substrate::metadata::decode_dispatch_error;
use parity_scale_codec::Decode;
use sp_runtime::DispatchError;

#[derive(Clone, Debug)]
pub enum UtilityEvent {
    ItemCompleted {
        extrinsic_index: Option<u32>,
    },
    ItemFailed {
        extrinsic_index: Option<u32>,
        dispatch_error: DispatchError,
    },
    BatchInterrupted {
        extrinsic_index: Option<u32>,
        item_index: u32,
        dispatch_error: DispatchError,
    },
    BatchCompleted {
        extrinsic_index: Option<u32>,
    },
    BatchCompletedWithErrors {
        extrinsic_index: Option<u32>,
    },
}

impl UtilityEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::ItemCompleted {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::ItemFailed {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::BatchInterrupted {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::BatchCompleted {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::BatchCompletedWithErrors {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl UtilityEvent {
    pub fn decode(
        runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "ItemCompleted" => Some(SubstrateEvent::Utility(UtilityEvent::ItemCompleted {
                extrinsic_index,
            })),
            "ItemFailed" => Some(SubstrateEvent::Utility(UtilityEvent::ItemFailed {
                extrinsic_index,
                dispatch_error: decode_dispatch_error(runtime_version, bytes)?,
            })),
            "BatchInterrupted" => Some(SubstrateEvent::Utility(UtilityEvent::BatchInterrupted {
                extrinsic_index,
                item_index: Decode::decode(bytes)?,
                dispatch_error: decode_dispatch_error(runtime_version, bytes)?,
            })),
            "BatchCompleted" => Some(SubstrateEvent::Utility(UtilityEvent::BatchCompleted {
                extrinsic_index,
            })),
            "BatchCompletedWithErrors" => Some(SubstrateEvent::Utility(
                UtilityEvent::BatchCompletedWithErrors { extrinsic_index },
            )),
            _ => None,
        };
        Ok(maybe_event)
    }
}
