use crate::{
    substrate::{
        argument::{Argument, ArgumentPrimitive},
        metadata::Metadata,
    },
    crypto::AccountId,
};
use frame_support::dispatch::{DispatchInfo, DispatchError};
use log::{debug};
use parity_scale_codec::{Compact, Decode, Input, Error};

#[derive(Debug)]
pub enum SubstrateEvent {
    // balances
    BalanceTransfer { extrinsic_index: u32, from: AccountId, to: AccountId, amount: u128 },
    BalanceDeposit { extrinsic_index: u32, account_id: AccountId, amount: u128 },
    // utility
    BatchItemCompleted { extrinsic_index: u32 },
    BatchInterrupted { extrinsic_index: u32, item_index: u32, dispatch_error: DispatchError },
    BatchCompleted { extrinsic_index: u32 },
    // system
    ExtrinsicSuccess { extrinsic_index: u32, dispatch_info: DispatchInfo },
    ExtrinsicFailed { extrinsic_index: u32, dispatch_error: DispatchError, dispatch_info: DispatchInfo },
    // staking

    // other - not interested
    Other { module_name: String, event_name: String, arguments: Vec<Argument> },
    /*

    Staking.EraPaid(EraIndex, Balance, Balance, )
    Staking.Withdrawn(AccountId, Balance, )
    Staking.Rewarded(AccountId, Balance, )
    Staking.Kicked(AccountId, AccountId, )
    Staking.StakingElectionFailed()
    Staking.PayoutStarted(EraIndex, AccountId, )
    Staking.OldSlashingReportDiscarded(SessionIndex, )
    Staking.StakersElected()
    Staking.Slashed(AccountId, Balance, )
    Staking.Bonded(AccountId, Balance, )
    Staking.Chilled(AccountId, )
    Staking.Unbonded(AccountId, Balance, )

    ImOnline.SomeOffline(Vec<IdentificationTuple>, )
    ImOnline.HeartbeatReceived(AuthorityId, )
    ImOnline.AllGood()

    Session.NewSession(SessionIndex, )

    Offences.Offence(Kind, OpaqueTimeSlot, )
     */
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum EventDecodeError {
    #[error("Decode error: {0}")]
    DecodeError(String),
}

impl From<parity_scale_codec::Error> for EventDecodeError {
    fn from(error: Error) -> Self {
        Self::DecodeError(error.to_string())
    }
}

impl SubstrateEvent {
    fn decode_event(metadata: &Metadata, bytes: &mut &[u8]) -> Result<Self, EventDecodeError> {
        use EventDecodeError::DecodeError;

        let phase = frame_system::Phase::decode(bytes)?;
        let maybe_extrinsic_index = match phase {
            frame_system::Phase::ApplyExtrinsic(extrinsic_index) => Some(extrinsic_index),
            _ => None
        };
        let module_index = bytes.read_byte()?;
        let event_index = bytes.read_byte()?;
        let module = metadata.modules.get(&module_index).unwrap();
        let event = module.events.get(&event_index).unwrap();
        // decode arguments
        debug!("{}.{}:", module.name, event.name);
        let mut arguments: Vec<Argument> = Vec::new();
        for argument_meta in &event.arguments {
            arguments.push(Argument::decode(argument_meta, &mut *bytes).unwrap());
        }
        // decode topics - unused
        let _topics = Vec::<sp_core::H256>::decode(bytes)?;

        let event_qualified_name = format!("{}.{}", module.name, event.name);
        let event = match event_qualified_name.as_str() {
            "System.ExtrinsicSuccess" => {
                let extrinsic_index = match maybe_extrinsic_index {
                    Some(extrinsic_index) => extrinsic_index,
                    _ => return Err(DecodeError("Cannot get extrinsic index from phase.".to_string()))
                };
                let argument_primitive = match &arguments[0] {
                    Argument::Primitive(argument_primitive) => *argument_primitive.clone(),
                    _ => return Err(DecodeError("Cannot get DispatchInfo argument primitive for ExtrinsicFailed.".to_string()))
                };
                let dispatch_info = match argument_primitive {
                    ArgumentPrimitive::DispatchInfo(dispatch_info) => dispatch_info,
                    _ => return Err(DecodeError("Cannot get DispatchInfo for ExtrinsicFailed.".to_string()))
                };
                Ok(SubstrateEvent::ExtrinsicSuccess { extrinsic_index, dispatch_info })
            }
            "System.ExtrinsicFailed" => {
                let extrinsic_index = match maybe_extrinsic_index {
                    Some(extrinsic_index) => extrinsic_index,
                    _ => return Err(DecodeError("Cannot get extrinsic index from phase.".to_string()))
                };
                let argument_primitive = match &arguments[0] {
                    Argument::Primitive(argument_primitive) => *argument_primitive.clone(),
                    _ => return Err(DecodeError("Cannot get DispatchInfo argument primitive for ExtrinsicFailed.".to_string()))
                };
                let dispatch_info = match argument_primitive {
                    ArgumentPrimitive::DispatchInfo(dispatch_info) => dispatch_info,
                    _ => return Err(DecodeError("Cannot get DispatchInfo for ExtrinsicFailed.".to_string()))
                };
                let argument_primitive = match &arguments[1] {
                    Argument::Primitive(argument_primitive) => *argument_primitive.clone(),
                    _ => return Err(DecodeError("Cannot get DispatchError argument primitive for ExtrinsicFailed.".to_string()))
                };
                let dispatch_error = match argument_primitive {
                    ArgumentPrimitive::DispatchError(dispatch_error) => dispatch_error,
                    _ => return Err(DecodeError("Cannot get DispatchInfo for ExtrinsicFailed.".to_string()))
                };
                Ok(SubstrateEvent::ExtrinsicFailed { extrinsic_index, dispatch_info, dispatch_error })
            }
            _ => {
                Ok(
                    SubstrateEvent::Other {
                        module_name: module.name.clone(),
                        event_name: event.name.clone(),
                        arguments,
                    }
                )
            }
        };
        event
    }

    pub fn decode_events(metadata: &Metadata, bytes: &mut &[u8]) -> anyhow::Result<Vec<Self>> {
        let event_count = <Compact<u32>>::decode(bytes)?.0;
        let mut events: Vec<Self> = Vec::with_capacity(event_count as usize);
        for _ in 0..event_count {
            events.push(SubstrateEvent::decode_event(metadata, bytes)?);
        }
        Ok(events)
    }
}