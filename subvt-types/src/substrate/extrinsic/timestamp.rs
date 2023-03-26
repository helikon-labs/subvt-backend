use crate::substrate::error::DecodeError;
use crate::substrate::extrinsic::{Signature, SubstrateExtrinsic};
use parity_scale_codec::{Compact, Decode};

const SET: &str = "set";

#[derive(Clone, Debug)]
pub enum TimestampExtrinsic {
    Set {
        maybe_signature: Option<Signature>,
        timestamp: u64,
    },
}

impl TimestampExtrinsic {
    pub fn decode(
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_event = match name {
            SET => {
                let moment: Compact<u64> = Decode::decode(bytes)?;
                Some(SubstrateExtrinsic::Timestamp(TimestampExtrinsic::Set {
                    maybe_signature: maybe_signature.clone(),
                    timestamp: moment.0,
                }))
            }
            _ => None,
        };
        Ok(maybe_event)
    }
}
