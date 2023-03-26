use crate::substrate::error::DecodeError;
use crate::substrate::extrinsic::{Signature, SubstrateExtrinsic};
use crate::substrate::BlockNumber;
use pallet_im_online::Heartbeat;
use parity_scale_codec::Decode;

const HEARTBEAT: &str = "heartbeat";

#[derive(Clone, Debug)]
pub enum ImOnlineExtrinsic {
    Hearbeat {
        maybe_signature: Option<Signature>,
        block_number: u32,
        session_index: u32,
        validator_index: u32,
    },
}

impl ImOnlineExtrinsic {
    pub fn decode(
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_event = match name {
            HEARTBEAT => {
                let heartbeat: Heartbeat<BlockNumber> = Decode::decode(bytes)?;
                Some(SubstrateExtrinsic::ImOnline(ImOnlineExtrinsic::Hearbeat {
                    maybe_signature: maybe_signature.clone(),
                    block_number: heartbeat.block_number,
                    session_index: heartbeat.session_index,
                    validator_index: heartbeat.authority_index,
                }))
            }
            _ => None,
        };
        Ok(maybe_event)
    }
}
