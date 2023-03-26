use crate::substrate::error::DecodeError;
use crate::substrate::extrinsic::{Signature, SubstrateExtrinsic};
use crate::substrate::Chain;
use frame_metadata::RuntimeMetadataV14;
use parity_scale_codec::{Compact, Decode};

const BATCH: &str = "batch";
const BATCH_ALL: &str = "batch_all";
const FORCE_BATCH: &str = "force_batch";

fn get_extrinsic_calls(
    chain: &Chain,
    runtime_version: u32,
    metadata: &RuntimeMetadataV14,
    maybe_signature: &Option<Signature>,
    bytes: &mut &[u8],
) -> Result<Vec<SubstrateExtrinsic>, DecodeError> {
    let call_count: Compact<u64> = Decode::decode(bytes)?;
    let mut calls = Vec::new();
    for _ in 0..call_count.0 {
        calls.push(SubstrateExtrinsic::decode_extrinsic(
            chain,
            runtime_version,
            metadata,
            maybe_signature,
            bytes,
        )?);
    }
    Ok(calls)
}

#[derive(Clone, Debug)]
pub enum UtilityExtrinsic {
    Batch {
        maybe_signature: Option<Signature>,
        calls: Vec<SubstrateExtrinsic>,
    },
    BatchAll {
        maybe_signature: Option<Signature>,
        calls: Vec<SubstrateExtrinsic>,
    },
    ForceBatch {
        maybe_signature: Option<Signature>,
        calls: Vec<SubstrateExtrinsic>,
    },
}

impl UtilityExtrinsic {
    pub fn decode(
        chain: &Chain,
        runtime_version: u32,
        metadata: &RuntimeMetadataV14,
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            BATCH => Some(SubstrateExtrinsic::Utility(UtilityExtrinsic::Batch {
                maybe_signature: maybe_signature.clone(),
                calls: get_extrinsic_calls(
                    chain,
                    runtime_version,
                    metadata,
                    maybe_signature,
                    bytes,
                )?,
            })),
            BATCH_ALL => Some(SubstrateExtrinsic::Utility(UtilityExtrinsic::BatchAll {
                maybe_signature: maybe_signature.clone(),
                calls: get_extrinsic_calls(
                    chain,
                    runtime_version,
                    metadata,
                    maybe_signature,
                    bytes,
                )?,
            })),
            FORCE_BATCH => Some(SubstrateExtrinsic::Utility(UtilityExtrinsic::ForceBatch {
                maybe_signature: maybe_signature.clone(),
                calls: get_extrinsic_calls(
                    chain,
                    runtime_version,
                    metadata,
                    maybe_signature,
                    bytes,
                )?,
            })),
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}
