//! Substrate extrinsic types, and decode logic.
//! Note: These are only the extrinsics that are utilized in SubVT.
use crate::substrate::metadata::decode_weight;
use crate::substrate::{Balance, Chain, RewardDestination};
use crate::{
    crypto::AccountId,
    substrate::{error::DecodeError, Block, MultiAddress, ProxyType, ValidatorPreferences},
};
use frame_metadata::RuntimeMetadataV14;
use frame_support::dispatch::Weight;
use pallet_im_online::Heartbeat;
use pallet_multisig::Timepoint;
use parity_scale_codec::{Compact, Decode, Input};
use polkadot_core_primitives::BlockNumber;
use sp_staking::EraIndex;

#[derive(Clone, Debug)]
pub enum MultisigExtrinsic {
    AsMulti {
        maybe_signature: Option<Signature>,
        threshold: u16,
        other_signatories: Vec<AccountId>,
        maybe_timepoint: Option<Timepoint<BlockNumber>>,
        call: Box<SubstrateExtrinsic>,
        store_call: bool,
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
            "as_multi" => Some(SubstrateExtrinsic::Multisig(MultisigExtrinsic::AsMulti {
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
                store_call: Decode::decode(bytes)?,
                max_weight: decode_weight(runtime_version, bytes)?,
            })),
            "as_multi_threshold_1" => Some(SubstrateExtrinsic::Multisig(
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

#[derive(Clone, Debug)]
pub enum ProxyExtrinsic {
    Proxy {
        maybe_signature: Option<Signature>,
        real: MultiAddress,
        force_proxy_type: Option<ProxyType>,
        call: Box<SubstrateExtrinsic>,
    },
    ProxyAnnounced {
        maybe_signature: Option<Signature>,
        delegate: MultiAddress,
        real: MultiAddress,
        force_proxy_type: Option<ProxyType>,
        call: Box<SubstrateExtrinsic>,
    },
}

impl ProxyExtrinsic {
    pub fn decode(
        chain: &Chain,
        runtime_version: u32,
        metadata: &RuntimeMetadataV14,
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            "proxy" => Some(SubstrateExtrinsic::Proxy(ProxyExtrinsic::Proxy {
                maybe_signature: maybe_signature.clone(),
                real: Decode::decode(bytes)?,
                force_proxy_type: Decode::decode(bytes)?,
                call: Box::new(SubstrateExtrinsic::decode_extrinsic(
                    chain,
                    runtime_version,
                    metadata,
                    maybe_signature,
                    bytes,
                )?),
            })),
            "proxy_announced" => Some(SubstrateExtrinsic::Proxy(ProxyExtrinsic::ProxyAnnounced {
                maybe_signature: maybe_signature.clone(),
                delegate: Decode::decode(bytes)?,
                real: Decode::decode(bytes)?,
                force_proxy_type: Decode::decode(bytes)?,
                call: Box::new(SubstrateExtrinsic::decode_extrinsic(
                    chain,
                    runtime_version,
                    metadata,
                    maybe_signature,
                    bytes,
                )?),
            })),
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}

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
            "heartbeat" => {
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
            "set" => {
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

#[derive(Clone, Debug)]
pub enum StakingExtrinsic {
    Bond {
        maybe_signature: Option<Signature>,
        controller: MultiAddress,
        amount: Balance,
        reward_destination: RewardDestination,
    },
    Nominate {
        maybe_signature: Option<Signature>,
        targets: Vec<MultiAddress>,
    },
    PayoutStakers {
        maybe_signature: Option<Signature>,
        validator_account_id: AccountId,
        era_index: EraIndex,
    },
    SetController {
        maybe_signature: Option<Signature>,
        controller: MultiAddress,
    },
    Validate {
        maybe_signature: Option<Signature>,
        preferences: ValidatorPreferences,
    },
}

impl StakingExtrinsic {
    pub fn decode(
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            "bond" => {
                let controller: MultiAddress = Decode::decode(bytes)?;
                let compact_amount: Compact<Balance> = Decode::decode(bytes)?;
                let reward_destination: RewardDestination = Decode::decode(bytes)?;
                Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Bond {
                    maybe_signature: maybe_signature.clone(),
                    controller,
                    amount: compact_amount.0,
                    reward_destination,
                }))
            }
            "nominate" => Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Nominate {
                maybe_signature: maybe_signature.clone(),
                targets: Decode::decode(bytes)?,
            })),
            "payout_stakers" => Some(SubstrateExtrinsic::Staking(
                StakingExtrinsic::PayoutStakers {
                    maybe_signature: maybe_signature.clone(),
                    validator_account_id: Decode::decode(bytes)?,
                    era_index: Decode::decode(bytes)?,
                },
            )),
            "set_controller" => Some(SubstrateExtrinsic::Staking(
                StakingExtrinsic::SetController {
                    maybe_signature: maybe_signature.clone(),
                    controller: Decode::decode(bytes)?,
                },
            )),
            "validate" => Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Validate {
                maybe_signature: maybe_signature.clone(),
                preferences: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}

#[derive(Clone, Debug)]
pub enum SessionExtrinsic {
    SetKeys {
        maybe_signature: Option<Signature>,
        session_keys: [u8; 192],
    },
}

impl SessionExtrinsic {
    pub fn decode(
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            "set_keys" => Some(SubstrateExtrinsic::Session(SessionExtrinsic::SetKeys {
                maybe_signature: maybe_signature.clone(),
                session_keys: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
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
            "batch" => Some(SubstrateExtrinsic::Utility(UtilityExtrinsic::Batch {
                maybe_signature: maybe_signature.clone(),
                calls: get_extrinsic_calls(
                    chain,
                    runtime_version,
                    metadata,
                    maybe_signature,
                    bytes,
                )?,
            })),
            "batch_all" => Some(SubstrateExtrinsic::Utility(UtilityExtrinsic::BatchAll {
                maybe_signature: maybe_signature.clone(),
                calls: get_extrinsic_calls(
                    chain,
                    runtime_version,
                    metadata,
                    maybe_signature,
                    bytes,
                )?,
            })),
            "force_batch" => Some(SubstrateExtrinsic::Utility(UtilityExtrinsic::ForceBatch {
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

#[derive(Clone, Debug)]
pub enum SubstrateExtrinsic {
    ImOnline(ImOnlineExtrinsic),
    Multisig(MultisigExtrinsic),
    Proxy(ProxyExtrinsic),
    Session(SessionExtrinsic),
    Staking(StakingExtrinsic),
    Timestamp(TimestampExtrinsic),
    Utility(UtilityExtrinsic),
    Other {
        module_name: String,
        call_name: String,
        signature: Option<Signature>,
    },
}

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
                let nonce: Compact<u32> = Decode::decode(&mut *bytes)?; // u32
                let tip: Compact<Balance> = Decode::decode(&mut *bytes)?;
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
            .types()
            .iter()
            .find(|metadata_type| metadata_type.id() == pallet.calls.clone().unwrap().ty.id())
            .unwrap();
        let call_variant = match calls_type.ty().type_def() {
            scale_info::TypeDef::Variant(variant) => variant
                .variants()
                .iter()
                .find(|variant| variant.index == call_index)
                .unwrap(),
            _ => {
                return Err(DecodeError::Error(format!(
                    "Unexpected non-variant call type: {:?}",
                    calls_type.ty().type_def()
                )))
            }
        };
        /*
        let pre_decode_bytes = <&[u8]>::clone(bytes);
        for call_field in call_variant.fields() {
            let call_field_type = get_metadata_type(metadata, call_field.ty().id());
            decode_field(metadata, call_field_type, bytes, false).unwrap();
        }
        let extrinsic_bytes_len = pre_decode_bytes.len() - bytes.len();
        let extrinsic_param_bytes = &mut &pre_decode_bytes[0..extrinsic_bytes_len];
        */
        let maybe_extrinsic = match pallet.name.as_str() {
            "ImOnline" => ImOnlineExtrinsic::decode(&call_variant.name, &maybe_signature, bytes)?,
            "Multisig" => MultisigExtrinsic::decode(
                chain,
                runtime_version,
                metadata,
                &call_variant.name,
                &maybe_signature,
                bytes,
            )?,
            "Staking" => StakingExtrinsic::decode(&call_variant.name, &maybe_signature, bytes)?,
            "Proxy" => ProxyExtrinsic::decode(
                chain,
                runtime_version,
                metadata,
                &call_variant.name,
                &maybe_signature,
                bytes,
            )?,
            "Session" => SessionExtrinsic::decode(&call_variant.name, &maybe_signature, bytes)?,
            "Timestamp" => TimestampExtrinsic::decode(&call_variant.name, &maybe_signature, bytes)?,
            "Utility" => UtilityExtrinsic::decode(
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
                    log::error!("{}", error_log);
                    result.push(Err(decode_error));
                }
            }
        }
        Ok(result)
    }
}
