use crate::crypto::AccountId;
use crate::substrate::error::DecodeError;
use crate::substrate::extrinsic::{Signature, SubstrateExtrinsic};
use crate::substrate::{BlockNumber, Chain};
use frame_metadata::RuntimeMetadataV14;
use frame_support::weights::Weight;
use pallet_multisig::Timepoint;
use parity_scale_codec::Decode;

const AS_MULTI: &str = "as_multi";
const AS_MULTI_THRESHOLD_1: &str = "as_multi_threshold_1";

#[derive(Clone, Debug)]
pub enum MultisigExtrinsic {
    AsMulti {
        maybe_signature: Option<Signature>,
        threshold: u16,
        other_signatories: Vec<AccountId>,
        maybe_timepoint: Option<Timepoint<BlockNumber>>,
        call: Box<SubstrateExtrinsic>,
        max_weight: Weight,
    },
    AsMultiThreshold1 {
        maybe_signature: Option<Signature>,
        other_signatories: Vec<AccountId>,
        call: Box<SubstrateExtrinsic>,
    },
}

impl MultisigExtrinsic {
    pub fn decode(
        chain: &Chain,
        runtime_version: u32,
        metadata: &RuntimeMetadataV14,
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            AS_MULTI => Some(SubstrateExtrinsic::Multisig(MultisigExtrinsic::AsMulti {
                maybe_signature: maybe_signature.clone(),
                threshold: Decode::decode(bytes)?,
                other_signatories: Decode::decode(bytes)?,
                maybe_timepoint: Decode::decode(bytes)?,
                call: Box::new(SubstrateExtrinsic::decode_extrinsic(
                    chain,
                    runtime_version,
                    metadata,
                    maybe_signature,
                    bytes,
                )?),
                max_weight: Decode::decode(bytes)?,
            })),
            AS_MULTI_THRESHOLD_1 => Some(SubstrateExtrinsic::Multisig(
                MultisigExtrinsic::AsMultiThreshold1 {
                    maybe_signature: maybe_signature.clone(),
                    other_signatories: Decode::decode(bytes)?,
                    call: Box::new(SubstrateExtrinsic::decode_extrinsic(
                        chain,
                        runtime_version,
                        metadata,
                        maybe_signature,
                        bytes,
                    )?),
                },
            )),
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}
