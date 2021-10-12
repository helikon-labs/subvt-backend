use crate::substrate::Chain;
use crate::{
    crypto::AccountId,
    substrate::{
        argument::{get_argument_primitive, get_argument_vector, Argument, ArgumentPrimitive},
        error::DecodeError,
        metadata::{ArgumentMeta, Metadata},
        Block, MultiAddress,
    },
};
use log::{debug, warn};
use parity_scale_codec::{Compact, Decode, Input};

#[derive(Clone, Debug)]
pub enum TimestampExtrinsic {
    Set {
        signature: Option<Signature>,
        timestamp: u64,
    },
}

impl TimestampExtrinsic {
    pub fn from(
        name: &str,
        signature: Option<Signature>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_event = match name {
            "set" => Some(SubstrateExtrinsic::Timestamp(TimestampExtrinsic::Set {
                signature,
                timestamp: get_argument_primitive!(&arguments[0], Moment).0,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Clone, Debug)]
pub enum StakingExtrinsic {
    Nominate {
        signature: Option<Signature>,
        targets: Vec<MultiAddress>,
    },
}

impl StakingExtrinsic {
    pub fn from(
        name: &str,
        signature: Option<Signature>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            "nominate" => Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Nominate {
                signature,
                targets: get_argument_vector!(&arguments[0], MultiAddress),
            })),
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}

#[derive(Clone, Debug)]
pub enum UtilityExtrinsic {
    Batch {
        signature: Option<Signature>,
        calls: Vec<SubstrateExtrinsic>,
    },
    BatchAll {
        signature: Option<Signature>,
        calls: Vec<SubstrateExtrinsic>,
    },
}

impl UtilityExtrinsic {
    pub fn from(
        name: &str,
        signature: Option<Signature>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            "batch" | "batch_all" => {
                let mut calls: Vec<SubstrateExtrinsic> = Vec::new();
                for argument in arguments {
                    calls.push(get_argument_primitive!(&argument, Call))
                }
                let extrinsic = if name == "batch" {
                    UtilityExtrinsic::Batch { signature, calls }
                } else {
                    UtilityExtrinsic::BatchAll { signature, calls }
                };
                Some(SubstrateExtrinsic::Utility(extrinsic))
            }
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}

#[derive(Clone, Debug)]
pub enum SubstrateExtrinsic {
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
    pub nonce: Option<u64>,
    pub tip: Option<u64>,
}

impl Signature {
    pub fn get_signer_account_id(&self) -> Option<AccountId> {
        self.signer.get_account_id()
    }
}

impl SubstrateExtrinsic {
    pub fn decode_extrinsic(
        chain: &Chain,
        metadata: &Metadata,
        skip_signature: bool,
        bytes: &mut &[u8],
    ) -> Result<Self, DecodeError> {
        let mut signature: Option<Signature> = None;
        if !skip_signature {
            let signed_version = bytes.read_byte()?;
            let sign_mask = 0b10000000;
            let version_mask = 0b00000100;
            let is_signed = (signed_version & sign_mask) == sign_mask;
            let _version = signed_version & version_mask;
            signature = if is_signed {
                let signer = if metadata.is_signer_address_multi(chain) {
                    MultiAddress::decode(&mut *bytes)?
                } else {
                    MultiAddress::Id(Decode::decode(&mut *bytes)?)
                };
                let signature = sp_runtime::MultiSignature::decode(&mut *bytes)?;
                let era: sp_runtime::generic::Era = Decode::decode(&mut *bytes)?;
                let nonce: Compact<u64> = Decode::decode(&mut *bytes)?;
                let tip: Compact<u64> = Decode::decode(&mut *bytes)?;
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
        let module = if let Some(module) = metadata.modules.get(&module_index) {
            module
        } else {
            return Err(DecodeError::Error(format!(
                "Cannot find module at index {}.",
                module_index
            )));
        };
        let call = if let Some(call) = module.calls.get(&call_index) {
            call
        } else {
            return Err(DecodeError::Error(format!(
                "Cannot find call at index {} for module {}.",
                call_index, module.name
            )));
        };
        let mut arguments: Vec<Argument> = Vec::new();
        for argument_meta in &call.arguments {
            if let ArgumentMeta::Vec(inner_argument_meta) = argument_meta {
                let inner_argument_meta = *inner_argument_meta.clone();
                if let ArgumentMeta::Primitive(name) = inner_argument_meta {
                    if name == "<T as Config>::Call" {
                        // skip signed signature - inner calls won't be signed
                        bytes.read_byte()?;
                        loop {
                            let extrinsic_result = SubstrateExtrinsic::decode_extrinsic(
                                chain,
                                metadata,
                                true,
                                &mut *bytes,
                            );
                            match extrinsic_result {
                                Ok(extrinsic) => arguments.push(Argument::Primitive(Box::new(
                                    ArgumentPrimitive::Call(extrinsic),
                                ))),
                                Err(_) => break,
                            }
                        }
                        continue;
                    }
                }
            }
            let argument = Argument::decode(chain, metadata, argument_meta, &mut *bytes)?;
            arguments.push(argument);
        }
        let maybe_extrinsic = match (module.name.as_str(), call.name.as_str()) {
            ("Timestamp", "set") => {
                TimestampExtrinsic::from(&call.name, signature.clone(), arguments.clone())?
            }
            ("Staking", "nominate") => {
                StakingExtrinsic::from(&call.name, signature.clone(), arguments.clone())?
            }
            ("Utility", "batch") | ("Utility", "batch_all") => {
                UtilityExtrinsic::from(&call.name, signature.clone(), arguments.clone())?
            }
            _ => None,
        };
        let extrinsic = if let Some(extrinsic) = maybe_extrinsic {
            debug!("Decoded extrinsic {}.{}.", module.name, call.name);
            extrinsic
        } else {
            warn!("Non-specified extrinsic {}.{}.", module.name, call.name);
            SubstrateExtrinsic::Other {
                signature,
                module_name: module.name.clone(),
                call_name: call.name.clone(),
            }
        };
        Ok(extrinsic)
    }

    pub fn decode_extrinsics(
        chain: &Chain,
        metadata: &Metadata,
        block: Block,
    ) -> anyhow::Result<Vec<Self>> {
        let mut extrinsics: Vec<Self> = Vec::new();
        for extrinsic_hex_string in block.extrinsics {
            let mut raw_bytes: &[u8] = &hex::decode(extrinsic_hex_string.trim_start_matches("0x"))?;
            let byte_vector: Vec<u8> = Decode::decode(&mut raw_bytes).unwrap();
            let mut bytes: &[u8] = byte_vector.as_ref();
            extrinsics.push(SubstrateExtrinsic::decode_extrinsic(
                chain, metadata, false, &mut bytes,
            )?);
        }
        Ok(extrinsics)
    }
}
