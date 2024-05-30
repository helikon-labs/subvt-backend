use crate::substrate::error::DecodeError;
use crate::substrate::extrinsic::{Signature, SubstrateExtrinsic};
use parity_scale_codec::Decode;

const SET_KEYS: &str = "set_keys";

#[derive(Clone, Debug)]
pub enum SessionExtrinsic {
    SetKeysLegacy {
        maybe_signature: Option<Signature>,
        session_keys: [u8; 192],
    },
    SetKeys {
        maybe_signature: Option<Signature>,
        session_keys: [u8; 193],
        proof: Vec<u8>,
    },
}

impl SessionExtrinsic {
    pub fn decode(
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            SET_KEYS => Some(SubstrateExtrinsic::Session(SessionExtrinsic::SetKeys {
                maybe_signature: maybe_signature.clone(),
                session_keys: Decode::decode(bytes)?,
                proof: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_extrinsic)
    }

    pub fn decode_legacy(
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            SET_KEYS => Some(SubstrateExtrinsic::Session(
                SessionExtrinsic::SetKeysLegacy {
                    maybe_signature: maybe_signature.clone(),
                    session_keys: Decode::decode(bytes)?,
                },
            )),
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}
