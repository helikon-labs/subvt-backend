use crate::substrate::error::DecodeError;
use crate::substrate::event::SubstrateEvent;
use crate::substrate::metadata::decode_dispatch_result;
use frame_support::dispatch::DispatchResult;

const PROXY_EXECUTED: &str = "ProxyExecuted";

#[derive(Clone, Debug)]
pub enum ProxyEvent {
    ProxyExecuted {
        extrinsic_index: Option<u32>,
        result: DispatchResult,
    },
}

impl ProxyEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::ProxyExecuted {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl ProxyEvent {
    pub fn decode(
        runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            PROXY_EXECUTED => Some(SubstrateEvent::Proxy(ProxyEvent::ProxyExecuted {
                extrinsic_index,
                result: decode_dispatch_result(runtime_version, bytes)?,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}
