//! Substrate extrinsic types, and decode logic.
//! Note: These are only the extrinsics that are utilized in SubVT.
use crate::substrate::metadata::{decode_field, get_metadata_type};
use crate::substrate::{Balance, Chain};
use crate::{
    crypto::AccountId,
    substrate::{error::DecodeError, Block, MultiAddress},
};
use frame_metadata::RuntimeMetadataV14;
use parity_scale_codec::{Compact, Decode, Input};

use staging_xcm::latest::Location;

pub mod conviction_voting;
pub mod multisig;
pub mod proxy;
pub mod session;
pub mod staking;
pub mod timestamp;
pub mod utility;

#[derive(Clone, Debug)]
pub struct Signature {
    pub signer: MultiAddress,
    pub signature: sp_runtime::MultiSignature,
    pub era: Option<sp_runtime::generic::Era>,
    pub nonce: Option<u32>,
    pub tip: Option<Balance>,
}

impl Signature {
    pub fn get_signer_account_id(&self) -> Option<AccountId> {
        self.signer.get_account_id()
    }
}

#[derive(Clone, Debug)]
pub enum SubstrateExtrinsic {
    ConvictionVoting(conviction_voting::ConvictionVotingExtrinsic),
    Multisig(multisig::MultisigExtrinsic),
    Proxy(proxy::ProxyExtrinsic),
    Session(session::SessionExtrinsic),
    Staking(staking::StakingExtrinsic),
    Timestamp(timestamp::TimestampExtrinsic),
    Utility(utility::UtilityExtrinsic),
    Other {
        module_name: String,
        call_name: String,
        signature: Option<Signature>,
    },
}

#[derive(Clone, Debug, Decode)]
pub struct TransactionPayment {
    pub tip: Compact<Balance>,
    pub asset_id: Option<Location>,
}

impl SubstrateExtrinsic {
    pub fn decode_extrinsic(
        chain: &Chain,
        runtime_version: u32,
        metadata: &RuntimeMetadataV14,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Self, DecodeError> {
        let mut maybe_signature = maybe_signature.clone();
        if maybe_signature.is_none() {
            let signed_version = bytes.read_byte()?;
            let sign_mask = 0b10000000;
            let version_mask = 0b00000100;
            let is_signed = (signed_version & sign_mask) == sign_mask;
            let _version = signed_version & version_mask;
            maybe_signature = if is_signed {
                let signer = MultiAddress::decode(&mut *bytes)?;
                let signature = sp_runtime::MultiSignature::decode(&mut *bytes)?;
                let era: sp_runtime::generic::Era = Decode::decode(&mut *bytes)?;
                let nonce: Compact<u32> = Decode::decode(&mut *bytes)?;
                let tip: Compact<Balance> = match chain {
                    Chain::KusamaAssetHub | Chain::PolkadotAssetHub => {
                        let payment: TransactionPayment = Decode::decode(&mut *bytes)?;
                        payment.tip
                    }
                    _ => Decode::decode(&mut *bytes)?,
                };
                let _extra: u8 = Decode::decode(&mut *bytes)?; // hash extension
                let signature = Signature {
                    signer,
                    signature,
                    era: Some(era),
                    nonce: Some(nonce.0),
                    tip: Some(tip.0),
                };
                Some(signature)
            } else {
                None
            };
        }
        let module_index: u8 = Decode::decode(&mut *bytes)?;
        let call_index: u8 = Decode::decode(&mut *bytes)?;
        let pallet = metadata
            .pallets
            .iter()
            .find(|metadata_pallet| metadata_pallet.index == module_index)
            .unwrap();
        let calls_type = metadata
            .types
            .types
            .iter()
            .find(|metadata_type| metadata_type.id == pallet.calls.clone().unwrap().ty.id)
            .unwrap();
        let call_variant = match &calls_type.ty.type_def {
            scale_info::TypeDef::Variant(variant) => variant
                .variants
                .iter()
                .find(|variant| variant.index == call_index)
                .unwrap(),
            _ => {
                return Err(DecodeError::Error(format!(
                    "Unexpected non-variant call type: {:?}",
                    calls_type.ty.type_def
                )))
            }
        };
        let maybe_extrinsic = match pallet.name.as_str() {
            "ConvictionVoting" => conviction_voting::ConvictionVotingExtrinsic::decode(
                &call_variant.name,
                &maybe_signature,
                bytes,
            )?,
            "Multisig" => multisig::MultisigExtrinsic::decode(
                chain,
                runtime_version,
                metadata,
                &call_variant.name,
                &maybe_signature,
                bytes,
            )?,
            "Staking" => {
                staking::StakingExtrinsic::decode(&call_variant.name, &maybe_signature, bytes)?
            }
            "Proxy" => proxy::ProxyExtrinsic::decode(
                chain,
                runtime_version,
                metadata,
                &call_variant.name,
                &maybe_signature,
                bytes,
            )?,
            /*"Session" => {
                if let Ok(extrinsic) =
                    session::SessionExtrinsic::decode(&call_variant.name, &maybe_signature, bytes)
                {
                    extrinsic
                } else {
                    session::SessionExtrinsic::decode_legacy(
                        &call_variant.name,
                        &maybe_signature,
                        bytes,
                    )?
                }
            }*/
            "Timestamp" => {
                timestamp::TimestampExtrinsic::decode(&call_variant.name, &maybe_signature, bytes)?
            }
            "Utility" => utility::UtilityExtrinsic::decode(
                chain,
                runtime_version,
                metadata,
                &call_variant.name,
                &maybe_signature,
                bytes,
            )?,
            _ => None,
        };
        let extrinsic = if let Some(extrinsic) = maybe_extrinsic {
            log::debug!("Decoded extrinsic {}.{}.", pallet.name, call_variant.name);
            extrinsic
        } else {
            for call_field in &call_variant.fields {
                let call_field_type = get_metadata_type(metadata, call_field.ty.id);
                decode_field(metadata, call_field_type, bytes, false).unwrap();
            }
            log::debug!(
                "Decoded non-specified extrinsic {}.{}.",
                pallet.name,
                call_variant.name
            );
            SubstrateExtrinsic::Other {
                module_name: pallet.name.clone(),
                call_name: call_variant.name.clone(),
                signature: maybe_signature,
            }
        };
        Ok(extrinsic)
    }

    pub fn decode_extrinsics(
        chain: &Chain,
        runtime_version: u32,
        metadata: &RuntimeMetadataV14,
        block_hash: &str,
        block: Block,
    ) -> anyhow::Result<Vec<Result<Self, DecodeError>>> {
        let mut result = Vec::new();
        for (extrinsic_index, extrinsic_hex_string) in block.extrinsics.iter().enumerate() {
            let mut raw_bytes: &[u8] = &hex::decode(extrinsic_hex_string.trim_start_matches("0x"))?;
            let byte_vector: Vec<u8> = Decode::decode(&mut raw_bytes).unwrap();
            let mut bytes: &[u8] = byte_vector.as_ref();
            match SubstrateExtrinsic::decode_extrinsic(
                chain,
                runtime_version,
                metadata,
                &None,
                &mut bytes,
            ) {
                Ok(extrinsic) => result.push(Ok(extrinsic)),
                Err(decode_error) => {
                    let error_log = match block.header.get_number() {
                        Ok(number) => format!(
                            "Error decoding extrinsic #{extrinsic_index} for block #{number}: {decode_error:?}",
                        ),
                        Err(error) => format!(
                            "[Cannot get block number: {error:?}] Error decoding extrinsic #{extrinsic_index} for block {block_hash}: {decode_error:?}",
                        ),
                    };
                    log::error!("{error_log}");
                    result.push(Err(decode_error));
                }
            }
        }
        Ok(result)
    }
}
