use crate::{argument::{Argument, ArgumentPrimitive}, metadata::Metadata};
use frame_support::dispatch::{DispatchInfo, DispatchError};
use log::{debug};
use parity_scale_codec::{Compact, Decode, Input, Error};
use subvt_types::crypto::AccountId;

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
                if let frame_system::Phase::ApplyExtrinsic(extrinsic_index) = phase {
                    if let Argument::Primitive(argument_primitive) = arguments[0].clone() {
                        if let ArgumentPrimitive::DispatchInfo(dispatch_info) = *argument_primitive {
                            Ok(
                                SubstrateEvent::ExtrinsicSuccess {
                                    extrinsic_index,
                                    dispatch_info,
                                }
                            )
                        } else {
                            Err(DecodeError("Cannot get DispatchInfo for ExtrinsicSuccess.".to_string()))
                        }
                    } else {
                        Err(DecodeError("Cannot get argument primitive for ExtrinsicSuccess.".to_string()))
                    }
                } else {
                    Err(DecodeError("Cannot get extrinsic index from phase.".to_string()))
                }
            }
            "System.ExtrinsicFailed" => {
                if let frame_system::Phase::ApplyExtrinsic(extrinsic_index) = phase {
                    let dispatch_info =
                        if let Argument::Primitive(argument_primitive) = arguments[0].clone() {
                            if let ArgumentPrimitive::DispatchInfo(dispatch_info) = *argument_primitive {
                                dispatch_info
                            } else {
                                return Err(DecodeError("Cannot get DispatchInfo for ExtrinsicFailed.".to_string()));
                            }
                        } else {
                            return Err(DecodeError("Cannot get argument primitive for ExtrinsicFailed.".to_string()));
                        };
                    let dispatch_error =
                        if let Argument::Primitive(argument_primitive) = arguments[1].clone() {
                            if let ArgumentPrimitive::DispatchError(dispatch_error) = *argument_primitive {
                                dispatch_error
                            } else {
                                return Err(DecodeError("Cannot get DispatchError for ExtrinsicFailed.".to_string()));
                            }
                        } else {
                            return Err(DecodeError("Cannot get argument primitive for ExtrinsicFailed.".to_string()));
                        };
                    Ok(
                        SubstrateEvent::ExtrinsicFailed {
                            extrinsic_index,
                            dispatch_info,
                            dispatch_error,
                        }
                    )
                } else {
                    Err(DecodeError("Cannot get extrinsic index from phase.".to_string()))
                }
            }
            _ => {
                Ok(
                    SubstrateEvent::Other {
                        module_name: module.name.clone(),
                        event_name: event.name.clone(),
                        arguments: Vec::new(),
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