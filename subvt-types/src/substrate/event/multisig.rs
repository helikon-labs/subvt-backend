use crate::crypto::AccountId;
use crate::substrate::error::DecodeError;
use crate::substrate::event::SubstrateEvent;
use crate::substrate::metadata::decode_dispatch_result;
use crate::substrate::{BlockNumber, CallHash};
use frame_support::dispatch::DispatchResult;
use pallet_multisig::Timepoint;
use parity_scale_codec::Decode;

const MULTISIG_EXECUTED: &str = "MultisigExecuted";

#[derive(Clone, Debug)]
pub enum MultisigEvent {
    MultisigExecuted {
        extrinsic_index: Option<u32>,
        approving_account_id: AccountId,
        timepoint: Timepoint<BlockNumber>,
        multisig_account_id: AccountId,
        call_hash: CallHash,
        result: DispatchResult,
    },
}

impl MultisigEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::MultisigExecuted {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl MultisigEvent {
    pub fn decode(
        runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            MULTISIG_EXECUTED => Some(SubstrateEvent::Multisig(MultisigEvent::MultisigExecuted {
                extrinsic_index,
                approving_account_id: Decode::decode(bytes)?,
                timepoint: Decode::decode(bytes)?,
                multisig_account_id: Decode::decode(bytes)?,
                call_hash: Decode::decode(bytes)?,
                result: decode_dispatch_result(runtime_version, bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}
