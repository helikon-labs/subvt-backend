//! Substrate event types, and decode logic.
//! Note: These are only the events that are utilized in SubVT.
use crate::substrate::{
    error::DecodeError,
    metadata::{decode_field, get_metadata_type},
    Block, Chain,
};
use frame_metadata::RuntimeMetadataV14;
use parity_scale_codec::{Compact, Decode};

pub mod democracy;
pub mod identity;
pub mod im_online;
pub mod multisig;
pub mod offences;
pub mod proxy;
pub mod referenda;
pub mod staking;
pub mod system;
pub mod utility;

#[derive(Clone, Debug)]
pub enum SubstrateEvent {
    Democracy(democracy::DemocracyEvent),
    Identity(identity::IdentityEvent),
    ImOnline(im_online::ImOnlineEvent),
    Multisig(multisig::MultisigEvent),
    Offences(offences::OffencesEvent),
    Proxy(proxy::ProxyEvent),
    Referenda(referenda::ReferendaEvent),
    Staking(staking::StakingEvent),
    System(system::SystemEvent),
    Utility(utility::UtilityEvent),
    Other {
        module_name: String,
        event_name: String,
        extrinsic_index: Option<u32>,
    },
}

impl SubstrateEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::Democracy(event) => event.get_extrinsic_index(),
            Self::Identity(event) => event.get_extrinsic_index(),
            Self::ImOnline(event) => event.get_extrinsic_index(),
            Self::Multisig(event) => event.get_extrinsic_index(),
            Self::Offences(event) => event.get_extrinsic_index(),
            Self::Proxy(event) => event.get_extrinsic_index(),
            Self::Referenda(event) => event.get_extrinsic_index(),
            Self::Staking(event) => event.get_extrinsic_index(),
            Self::System(event) => event.get_extrinsic_index(),
            Self::Utility(event) => event.get_extrinsic_index(),
            Self::Other {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl SubstrateEvent {
    fn decode_event(
        _chain: &Chain,
        metadata: &RuntimeMetadataV14,
        runtime_version: u32,
        bytes: &mut &[u8],
    ) -> Result<Self, DecodeError> {
        let phase = frame_system::Phase::decode(bytes)?;
        let extrinsic_index = match phase {
            frame_system::Phase::ApplyExtrinsic(extrinsic_index) => Some(extrinsic_index),
            _ => None,
        };
        let module_index: u8 = Decode::decode(&mut *bytes)?;
        let event_index: u8 = Decode::decode(&mut *bytes)?;
        let pallet = metadata
            .pallets
            .iter()
            .find(|p| p.index == module_index)
            .unwrap();
        let event_type = metadata
            .types
            .types
            .iter()
            .find(|ty| ty.id == pallet.event.clone().unwrap().ty.id)
            .unwrap();
        let event_variant = match &event_type.ty.type_def {
            scale_info::TypeDef::Variant(variant) => variant
                .variants
                .iter()
                .find(|variant| variant.index == event_index)
                .unwrap(),
            _ => {
                return Err(DecodeError::Error(format!(
                    "Unexpected non-variant event type: {:?}",
                    event_type.ty.type_def
                )))
            }
        };
        let pre_event_bytes = <&[u8]>::clone(bytes);
        // decode parameters
        for event_field in &event_variant.fields {
            let event_field_type = get_metadata_type(metadata, event_field.ty.id);
            decode_field(metadata, event_field_type, bytes, false).unwrap();
        }
        // post bytes :: get bytes => decode by runtime
        let event_bytes_len = pre_event_bytes.len() - bytes.len();
        let event_bytes = &mut &pre_event_bytes[0..event_bytes_len];
        // decode topics - unused
        let _topics = Vec::<sp_core::H256>::decode(bytes)?;
        // decode events
        let maybe_event = match pallet.name.as_str() {
            "Democracy" => democracy::DemocracyEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            "Identity" => identity::IdentityEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            "ImOnline" => im_online::ImOnlineEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            "Multisig" => multisig::MultisigEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            "Offences" => offences::OffencesEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            "Proxy" => proxy::ProxyEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            "Referenda" => referenda::ReferendaEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            "Staking" => staking::StakingEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            "System" => system::SystemEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            "Utility" => utility::UtilityEvent::decode(
                runtime_version,
                &event_variant.name,
                extrinsic_index,
                event_bytes,
            )?,
            _ => None,
        };
        let substrate_event = if let Some(substrate_event) = maybe_event {
            log::debug!("Decoded event {}.{}.", pallet.name, event_variant.name);
            substrate_event
        } else {
            log::debug!(
                "Decoded non-specified event {}.{}.",
                pallet.name,
                event_variant.name
            );
            SubstrateEvent::Other {
                module_name: pallet.name.clone(),
                event_name: event_variant.name.clone(),
                extrinsic_index,
            }
        };
        Ok(substrate_event)
    }

    pub fn decode_events(
        chain: &Chain,
        metadata: &RuntimeMetadataV14,
        runtime_version: u32,
        block_hash: &str,
        block: Block,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Vec<Result<Self, DecodeError>>> {
        let event_count = <Compact<u32>>::decode(bytes)?.0;
        let mut result = Vec::with_capacity(event_count as usize);
        for event_index in 0..event_count {
            match SubstrateEvent::decode_event(chain, metadata, runtime_version, bytes) {
                Ok(event) => result.push(Ok(event)),
                Err(decode_error) => {
                    let error_log = match block.header.get_number() {
                        Ok(number) => format!(
                            "Error decoding event #{event_index} for block #{number}: {decode_error:?}",
                        ),
                        Err(error) => format!(
                            "[Cannot get block number: {error:?}] Error decoding extrinsic #{event_index} for block {block_hash}: {decode_error:?}",
                        ),
                    };
                    log::error!("{}", error_log);
                    result.push(Err(decode_error));
                    break;
                }
            }
        }
        Ok(result)
    }
}
