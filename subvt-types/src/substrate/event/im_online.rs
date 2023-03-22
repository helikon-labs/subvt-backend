use crate::substrate::error::DecodeError;
use crate::substrate::event::SubstrateEvent;
use crate::{crypto::AccountId, substrate::Balance};
use pallet_staking::Exposure;
use parity_scale_codec::Decode;

pub type IdentificationTuple = (AccountId, Exposure<AccountId, Balance>);

#[derive(Clone, Debug)]
pub enum ImOnlineEvent {
    AllGood {
        extrinsic_index: Option<u32>,
    },
    HeartbeatReceived {
        extrinsic_index: Option<u32>,
        im_online_key_hex_string: String,
    },
    SomeOffline {
        identification_tuples: Vec<IdentificationTuple>,
    },
}

impl ImOnlineEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::AllGood {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::HeartbeatReceived {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::SomeOffline { .. } => None,
        }
    }
}

impl ImOnlineEvent {
    pub fn decode(
        _runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "AllGood" => Some(SubstrateEvent::ImOnline(ImOnlineEvent::AllGood {
                extrinsic_index,
            })),
            "HeartbeatReceived" => {
                let im_online_key: pallet_im_online::sr25519::AuthorityId = Decode::decode(bytes)?;
                let im_online_key_bytes: &[u8] = im_online_key.as_ref();
                Some(SubstrateEvent::ImOnline(ImOnlineEvent::HeartbeatReceived {
                    extrinsic_index,
                    im_online_key_hex_string: format!(
                        "0x{}",
                        hex::encode_upper(im_online_key_bytes)
                    ),
                }))
            }
            "SomeOffline" => Some(SubstrateEvent::ImOnline(ImOnlineEvent::SomeOffline {
                identification_tuples: Decode::decode(bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}
