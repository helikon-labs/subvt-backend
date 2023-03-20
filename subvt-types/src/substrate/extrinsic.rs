//! Substrate extrinsic types, and decode logic.
//! Note: These are only the extrinsics that are utilized in SubVT.
use crate::substrate::{Balance, Chain, RewardDestination};
use crate::{
    crypto::AccountId,
    substrate::{error::DecodeError, Block, MultiAddress, ProxyType, ValidatorPreferences},
};
use frame_metadata::RuntimeMetadataV14;
use pallet_multisig::Timepoint;
use parity_scale_codec::{Compact, Decode, Input};
use polkadot_core_primitives::BlockNumber;

type EraIndex = u32;

#[derive(Clone, Debug)]
pub enum MultisigExtrinsic {
    AsMulti {
        maybe_signature: Option<Signature>,
        threshold: u16,
        other_signatories: Vec<AccountId>,
        maybe_timepoint: Option<Timepoint<BlockNumber>>,
        call: Box<SubstrateExtrinsic>,
        store_call: bool,
        max_weight: frame_support::dispatch::Weight,
    },
    AsMultiThreshold1 {
        maybe_signature: Option<Signature>,
        other_signatories: Vec<AccountId>,
        call: Box<SubstrateExtrinsic>,
    },
}

impl MultisigExtrinsic {
    pub fn from(
        _name: &str,
        _maybe_signature: Option<Signature>,
        _bytes: &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        /*
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
                    maybe_signature,
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
                        maybe_signature,
                        other_signatories: get_argument_vector!(&arguments[0], AccountId),
                        call: Box::new(get_argument_primitive!(&arguments[1], Call)),
                    },
                ))
            }
            _ => None,
        };
        Ok(maybe_extrinsic)
         */
        Ok(None)
    }
}

#[derive(Clone, Debug)]
pub enum ProxyExtrinsic {
    Proxy {
        maybe_signature: Option<Signature>,
        real_account_id: AccountId,
        force_proxy_type: Option<ProxyType>,
        call: Box<SubstrateExtrinsic>,
    },
    ProxyAnnounced {
        maybe_signature: Option<Signature>,
        delegate_account_id: AccountId,
        real_account_id: AccountId,
        force_proxy_type: Option<ProxyType>,
        call: Box<SubstrateExtrinsic>,
    },
}

impl ProxyExtrinsic {
    pub fn from(
        _name: &str,
        _maybe_signature: Option<Signature>,
        _bytes: &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        /*
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
                    maybe_signature,
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
                    maybe_signature,
                    delegate_account_id: get_argument_primitive!(&arguments[0], AccountId),
                    real_account_id: get_argument_primitive!(&arguments[1], AccountId),
                    force_proxy_type: get_optional_argument_primitive!(&arguments[2], ProxyType),
                    call: Box::new(get_argument_primitive!(&arguments[3], Call)),
                }))
            }
            _ => None,
        };
        Ok(maybe_extrinsic)
         */
        Ok(None)
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
    pub fn from(
        _name: &str,
        _maybe_signature: Option<Signature>,
        _bytes: &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        /*
        let maybe_event = match name {
            "heartbeat" => {
                let heartbeat = get_argument_primitive!(&arguments[0], Heartbeat);
                Some(SubstrateExtrinsic::ImOnline(ImOnlineExtrinsic::Hearbeat {
                    maybe_signature,
                    block_number: heartbeat.block_number,
                    session_index: heartbeat.session_index,
                    validator_index: heartbeat.authority_index,
                }))
            }
            _ => None,
        };
        Ok(maybe_event)
         */
        Ok(None)
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
    pub fn from(
        _name: &str,
        _maybe_signature: Option<Signature>,
        _bytes: &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        /*
        let maybe_event = match name {
            "set" => Some(SubstrateExtrinsic::Timestamp(TimestampExtrinsic::Set {
                maybe_signature,
                timestamp: get_argument_primitive!(&arguments[0], CompactMoment).0,
            })),
            _ => None,
        };
        Ok(maybe_event)
         */
        Ok(None)
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
    pub fn from(
        _name: &str,
        _maybe_signature: Option<Signature>,
        _bytes: &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        /*
        let maybe_extrinsic = match name {
            "bond" => Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Bond {
                maybe_signature,
                controller: get_argument_primitive!(&arguments[0], MultiAddress),
                amount: get_argument_primitive!(&arguments[1], CompactBalance).0,
                reward_destination: get_argument_primitive!(&arguments[2], RewardDestination),
            })),
            "nominate" => Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Nominate {
                maybe_signature,
                targets: get_argument_vector!(&arguments[0], MultiAddress),
            })),
            "payout_stakers" => Some(SubstrateExtrinsic::Staking(
                StakingExtrinsic::PayoutStakers {
                    maybe_signature,
                    validator_account_id: get_argument_primitive!(&arguments[0], AccountId),
                    era_index: get_argument_primitive!(&arguments[1], EraIndex),
                },
            )),
            "set_controller" => Some(SubstrateExtrinsic::Staking(
                StakingExtrinsic::SetController {
                    maybe_signature,
                    controller: get_argument_primitive!(&arguments[0], MultiAddress),
                },
            )),
            "validate" => Some(SubstrateExtrinsic::Staking(StakingExtrinsic::Validate {
                maybe_signature,
                preferences: get_argument_primitive!(&arguments[0], ValidatorPreferences),
            })),
            _ => None,
        };
        Ok(maybe_extrinsic)
         */
        Ok(None)
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
    pub fn from(
        _name: &str,
        _maybe_signature: Option<Signature>,
        _bytes: &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        /*
        let maybe_extrinsic = match name {
            "set_keys" => Some(SubstrateExtrinsic::Session(SessionExtrinsic::SetKeys {
                maybe_signature,
                session_keys: get_argument_primitive!(&arguments[0], SessionKeys),
            })),
            _ => None,
        };
        Ok(maybe_extrinsic)
         */
        Ok(None)
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

impl UtilityExtrinsic {
    pub fn from(
        _name: &str,
        _maybe_signature: Option<Signature>,
        _bytes: &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        /*
        let maybe_extrinsic = match name {
            "batch" => {
                let mut calls: Vec<SubstrateExtrinsic> = Vec::new();
                for call in get_argument_vector!(&arguments[0], Call) {
                    calls.push(call);
                }
                Some(SubstrateExtrinsic::Utility(UtilityExtrinsic::Batch {
                    maybe_signature,
                    calls,
                }))
            }
            "batch_all" => {
                let mut calls: Vec<SubstrateExtrinsic> = Vec::new();
                for call in get_argument_vector!(&arguments[0], Call) {
                    calls.push(call);
                }
                Some(SubstrateExtrinsic::Utility(UtilityExtrinsic::BatchAll {
                    maybe_signature,
                    calls,
                }))
            }
            "force_batch" => {
                let mut calls: Vec<SubstrateExtrinsic> = Vec::new();
                for call in get_argument_vector!(&arguments[0], Call) {
                    calls.push(call);
                }
                Some(SubstrateExtrinsic::Utility(UtilityExtrinsic::ForceBatch {
                    maybe_signature,
                    calls,
                }))
            }
            _ => None,
        };
        Ok(maybe_extrinsic)
         */
        Ok(None)
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
        _chain: &Chain,
        _runtime_version: u32,
        _metadata: &RuntimeMetadataV14,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Self, DecodeError> {
        let mut signature = maybe_signature.clone();
        if signature.is_none() {
            let signed_version = bytes.read_byte()?;
            let sign_mask = 0b10000000;
            let version_mask = 0b00000100;
            let is_signed = (signed_version & sign_mask) == sign_mask;
            let _version = signed_version & version_mask;
            signature = if is_signed {
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
        let _module_index: u8 = Decode::decode(&mut *bytes)?;
        let _call_index: u8 = Decode::decode(&mut *bytes)?;

        /*
        let module = if let Some(module) = metadata.modules.get(&module_index) {
            module
        } else {
            return Err(DecodeError::Error(format!(
                "Cannot find module at index {module_index}.",
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
            ("ImOnline", "heartbeat") => {
                ImOnlineExtrinsic::from(&call.name, signature.clone(), bytes)?
            }
            ("Multisig", "as_multi") | ("Multisig", "as_multi_threshold_1") => {
                MultisigExtrinsic::from(&call.name, signature.clone(), bytes)?
            }
            ("Staking", "bond")
            | ("Staking", "nominate")
            | ("Staking", "payout_stakers")
            | ("Staking", "validate")
            | ("Staking", "set_controller") => {
                StakingExtrinsic::from(&call.name, signature.clone(), bytes)?
            }
            ("Proxy", "proxy") | ("Proxy", "proxy_announced") => {
                ProxyExtrinsic::from(&call.name, signature.clone(), bytes)?
            }
            ("Session", "set_keys") => {
                SessionExtrinsic::from(&call.name, signature.clone(), bytes)?
            }
            ("Timestamp", "set") => TimestampExtrinsic::from(&call.name, signature.clone(), bytes)?,
            ("Utility", "batch") | ("Utility", "batch_all") | ("Utility", "force_batch") => {
                UtilityExtrinsic::from(&call.name, signature.clone(), bytes)?
            }
            _ => None,
        };
        let extrinsic = if let Some(extrinsic) = maybe_extrinsic {
            log::debug!("Decoded extrinsic {}.{}.", module.name, call.name);
            extrinsic
        } else {
            log::debug!(
                "Decoded non-specified extrinsic {}.{}.",
                module.name,
                call.name
            );
            SubstrateExtrinsic::Other {
                signature,
                module_name: module.name.clone(),
                call_name: call.name.clone(),
            }
        };
        Ok(extrinsic)
         */
        Ok(SubstrateExtrinsic::Other {
            module_name: "asd".to_string(),
            call_name: "dsa".to_string(),
            signature,
        })
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
