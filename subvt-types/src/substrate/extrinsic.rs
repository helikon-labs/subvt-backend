use crate::substrate::Chain;
use crate::{
    crypto::AccountId,
    substrate::{
        argument::{
            get_argument_primitive, get_argument_vector, get_optional_argument_primitive, Argument,
            ArgumentPrimitive,
        },
        error::DecodeError,
        metadata::{ArgumentMeta, Metadata},
        Block, MultiAddress, ProxyType, ValidatorPreferences,
    },
};
use log::{debug, error};
use pallet_multisig::Timepoint;
use parity_scale_codec::{Compact, Decode, Input};
use polkadot_core_primitives::BlockNumber;

#[derive(Clone, Debug)]
pub enum MultisigExtrinsic {
    AsMulti {
        signature: Option<Signature>,
        threshold: u16,
        other_signatories: Vec<AccountId>,
        maybe_timepoint: Option<Timepoint<BlockNumber>>,
        call: Box<SubstrateExtrinsic>,
        store_call: bool,
        max_weight: u64,
    },
    AsMultiThreshold1 {
        other_signatories: Vec<AccountId>,
        call: Box<SubstrateExtrinsic>,
    },
}

impl MultisigExtrinsic {
    pub fn from(
        name: &str,
        signature: Option<Signature>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            "as_multi" => {
                if arguments.len() < 6 {
                    return Err(
                        DecodeError::Error(
                            format!(
                                "Cannot decode Multisign.as_multi extrinsic. Not enough parameters. Expected 6, found {}.",
                                arguments.len()
                            )
                        )
                    );
                }
                Some(SubstrateExtrinsic::Multisig(MultisigExtrinsic::AsMulti {
                    signature,
                    threshold: get_argument_primitive!(&arguments[0], U16),
                    other_signatories: get_argument_vector!(&arguments[1], AccountId),
                    maybe_timepoint: get_optional_argument_primitive!(
                        &arguments[2],
                        MultisigTimepoint
                    ),
                    call: Box::new(get_argument_primitive!(&arguments[3], Call)),
                    store_call: get_argument_primitive!(&arguments[4], Bool),
                    max_weight: get_argument_primitive!(&arguments[5], Weight),
                }))
            }
            "as_multi_threshold_1" => {
                if arguments.len() < 2 {
                    return Err(
                        DecodeError::Error(
                            format!(
                                "Cannot decode Multisig.as_multi extrinsic. Not enough parameters. Expected 2, found {}.",
                                arguments.len()
                            )
                        )
                    );
                }
                Some(SubstrateExtrinsic::Multisig(
                    MultisigExtrinsic::AsMultiThreshold1 {
                        other_signatories: get_argument_vector!(&arguments[0], AccountId),
                        call: Box::new(get_argument_primitive!(&arguments[1], Call)),
                    },
                ))
            }
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}

#[derive(Clone, Debug)]
pub enum ProxyExtrinsic {
    Proxy {
        signature: Option<Signature>,
        real_account_id: AccountId,
        force_proxy_type: Option<ProxyType>,
        call: Box<SubstrateExtrinsic>,
    },
    ProxyAnnounced {
        signature: Option<Signature>,
        delegate_account_id: AccountId,
        real_account_id: AccountId,
        force_proxy_type: Option<ProxyType>,
        call: Box<SubstrateExtrinsic>,
    },
}

impl ProxyExtrinsic {
    pub fn from(
        name: &str,
        signature: Option<Signature>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_extrinsic = match name {
            "proxy" => {
                if arguments.len() < 3 {
                    return Err(
                        DecodeError::Error(
                            format!(
                                "Cannot decode Proxy.proxy extrinsic. Not enough parameters. Expected 3, found {}.",
                                arguments.len()
                            )
                        )
                    );
                }
                Some(SubstrateExtrinsic::Proxy(ProxyExtrinsic::Proxy {
                    signature,
                    real_account_id: get_argument_primitive!(&arguments[0], AccountId),
                    force_proxy_type: get_optional_argument_primitive!(&arguments[1], ProxyType),
                    call: Box::new(get_argument_primitive!(&arguments[2], Call)),
                }))
            }
            "proxy_announced" => {
                if arguments.len() < 4 {
                    return Err(
                        DecodeError::Error(
                            format!(
                                "Cannot decode Proxy.proxy extrinsic. Not enough parameters. Expected 4, found {}.",
                                arguments.len()
                            )
                        )
                    );
                }
                Some(SubstrateExtrinsic::Proxy(ProxyExtrinsic::ProxyAnnounced {
                    signature,
                    delegate_account_id: get_argument_primitive!(&arguments[0], AccountId),
                    real_account_id: get_argument_primitive!(&arguments[1], AccountId),
                    force_proxy_type: get_optional_argument_primitive!(&arguments[2], ProxyType),
                    call: Box::new(get_argument_primitive!(&arguments[3], Call)),
                }))
            }
            _ => None,
        };
        Ok(maybe_extrinsic)
    }
}

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
                timestamp: get_argument_primitive!(&arguments[0], CompactMoment).0,
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
    Validate {
        signature: Option<Signature>,
        preferences: ValidatorPreferences,
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
            "validate" => Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Validate {
                signature,
                preferences: get_argument_primitive!(&arguments[0], ValidatorPreferences),
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
                for call in get_argument_vector!(&arguments[0], Call) {
                    calls.push(call);
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
    Multisig(MultisigExtrinsic),
    Proxy(ProxyExtrinsic),
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
        signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Self, DecodeError> {
        let mut signature = signature.clone();
        if signature.is_none() {
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
                // vector of calls should be handled differently,
                // the encoding doesn't include the length of the calls as it would in the case
                // of any other vector encoding, they're just concat.ed one after another
                let inner_argument_meta = *inner_argument_meta.clone();
                if let ArgumentMeta::Primitive(name) = inner_argument_meta {
                    if name == "<T as Config>::Call" {
                        let mut call_args: Vec<Argument> = Vec::new();
                        // skip signed signature - inner calls won't be signed
                        let call_count: Compact<u64> = Decode::decode(&mut *bytes).unwrap();
                        for _ in 0..call_count.0 {
                            let extrinsic_result = SubstrateExtrinsic::decode_extrinsic(
                                chain,
                                metadata,
                                &signature,
                                &mut *bytes,
                            );
                            match extrinsic_result {
                                Ok(extrinsic) => call_args.push(Argument::Primitive(Box::new(
                                    ArgumentPrimitive::Call(extrinsic),
                                ))),
                                Err(error) => return Err(error),
                            }
                        }
                        arguments.push(Argument::Vec(call_args));
                        continue;
                    }
                }
            }
            let argument =
                Argument::decode(chain, metadata, argument_meta, &signature, &mut *bytes)?;
            arguments.push(argument);
        }
        let maybe_extrinsic = match (module.name.as_str(), call.name.as_str()) {
            ("Multisig", "as_multi") | ("Multisig", "as_multi_threshold_1") => {
                MultisigExtrinsic::from(&call.name, signature.clone(), arguments.clone())?
            }
            ("Staking", "nominate") | ("Staking", "validate") => {
                StakingExtrinsic::from(&call.name, signature.clone(), arguments.clone())?
            }
            ("Proxy", "proxy") | ("Proxy", "proxy_announced") => {
                ProxyExtrinsic::from(&call.name, signature.clone(), arguments.clone())?
            }
            ("Timestamp", "set") => {
                TimestampExtrinsic::from(&call.name, signature.clone(), arguments.clone())?
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
            debug!(
                "Decoded non-specified extrinsic {}.{}.",
                module.name, call.name
            );
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
        for (extrinsic_index, extrinsic_hex_string) in block.extrinsics.iter().enumerate() {
            let mut raw_bytes: &[u8] = &hex::decode(extrinsic_hex_string.trim_start_matches("0x"))?;
            let byte_vector: Vec<u8> = Decode::decode(&mut raw_bytes).unwrap();
            let mut bytes: &[u8] = byte_vector.as_ref();
            match SubstrateExtrinsic::decode_extrinsic(chain, metadata, &None, &mut bytes) {
                Ok(extrinsic) => extrinsics.push(extrinsic),
                Err(error) => error!(
                    "Error decoding extrinsic #{} for block #{}: {:?}",
                    extrinsic_index,
                    block.header.get_number().unwrap(),
                    error
                ),
            }
        }
        Ok(extrinsics)
    }
}
