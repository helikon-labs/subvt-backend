use crate::substrate::error::DecodeError;
use crate::substrate::extrinsic::{Signature, SubstrateExtrinsic};
use crate::substrate::{Chain, MultiAddress, ProxyType};
use frame_metadata::RuntimeMetadataV14;
use parity_scale_codec::Decode;

const PROXY: &str = "proxy";
const PROXY_ANNOUNCED: &str = "proxy_announced";

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
            PROXY => Some(SubstrateExtrinsic::Proxy(ProxyExtrinsic::Proxy {
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
            PROXY_ANNOUNCED => Some(SubstrateExtrinsic::Proxy(ProxyExtrinsic::ProxyAnnounced {
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
