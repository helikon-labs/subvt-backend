use crate::{
    crypto::AccountId,
    substrate::{
        argument::{
            get_argument_primitive, get_argument_vector, Argument, ArgumentPrimitive,
            IdentificationTuple,
        },
        error::DecodeError,
        metadata::Metadata,
        Balance, Block, Chain, OpaqueTimeSlot,
    },
};
use frame_support::dispatch::{DispatchError, DispatchInfo};
use log::{debug, error};
use pallet_identity::RegistrarIndex;
use pallet_staking::EraIndex;
use parity_scale_codec::{Compact, Decode};
use polkadot_primitives::v1::{CandidateReceipt, CoreIndex, GroupIndex, HeadData, Id};
use sp_staking::offence::Kind;
use sp_staking::SessionIndex;

#[derive(Debug)]
pub enum BalancesEvent {
    BalanceSet {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        free_amount: Balance,
        reserved_amount: Balance,
    },
    Deposit {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        amount: Balance,
    },
    Transfer {
        extrinsic_index: Option<u32>,
        from_account_id: AccountId,
        to_account_id: AccountId,
        amount: Balance,
    },
}

impl BalancesEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "BalanceSet" => Some(SubstrateEvent::Balances(BalancesEvent::BalanceSet {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                free_amount: get_argument_primitive!(&arguments[1], Balance),
                reserved_amount: get_argument_primitive!(&arguments[2], Balance),
            })),
            "Deposit" => Some(SubstrateEvent::Balances(BalancesEvent::Deposit {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                amount: get_argument_primitive!(&arguments[1], Balance),
            })),
            "Transfer" => Some(SubstrateEvent::Balances(BalancesEvent::Transfer {
                extrinsic_index,
                from_account_id: get_argument_primitive!(&arguments[0], AccountId),
                to_account_id: get_argument_primitive!(&arguments[1], AccountId),
                amount: get_argument_primitive!(&arguments[2], Balance),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum IdentityEvent {
    IdentityCleared {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        returned_balance: Balance,
    },
    IdentityKilled {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        slashed_balance: Balance,
    },
    IdentitySet {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
    },
    JudgementGiven {
        extrinsic_index: Option<u32>,
        target_account_id: AccountId,
        registrar_index: RegistrarIndex,
    },
    JudgementRequested {
        extrinsic_index: Option<u32>,
        target_account_id: AccountId,
        registrar_index: RegistrarIndex,
    },
    JudgementUnrequested {
        extrinsic_index: Option<u32>,
        target_account_id: AccountId,
        registrar_index: RegistrarIndex,
    },
    SubIdentityAdded {
        extrinsic_index: Option<u32>,
        sub_account_id: AccountId,
        main_account_id: AccountId,
        deposit: Balance,
    },
    SubIdentityRemoved {
        extrinsic_index: Option<u32>,
        sub_account_id: AccountId,
        main_account_id: AccountId,
        freed_deposit: Balance,
    },
    SubIdentityRevoked {
        extrinsic_index: Option<u32>,
        sub_account_id: AccountId,
        main_account_id: AccountId,
        repatriated_deposit: Balance,
    },
}

impl IdentityEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "IdentityCleared" => Some(SubstrateEvent::Identity(IdentityEvent::IdentityCleared {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                returned_balance: get_argument_primitive!(&arguments[1], Balance),
            })),
            "IdentityKilled" => Some(SubstrateEvent::Identity(IdentityEvent::IdentityKilled {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                slashed_balance: get_argument_primitive!(&arguments[1], Balance),
            })),
            "IdentitySet" => Some(SubstrateEvent::Identity(IdentityEvent::IdentitySet {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
            })),
            "JudgementGiven" => Some(SubstrateEvent::Identity(IdentityEvent::JudgementGiven {
                extrinsic_index,
                target_account_id: get_argument_primitive!(&arguments[0], AccountId),
                registrar_index: get_argument_primitive!(&arguments[1], RegistrarIndex),
            })),
            "JudgementRequested" => Some(SubstrateEvent::Identity(
                IdentityEvent::JudgementRequested {
                    extrinsic_index,
                    target_account_id: get_argument_primitive!(&arguments[0], AccountId),
                    registrar_index: get_argument_primitive!(&arguments[1], RegistrarIndex),
                },
            )),
            "JudgementUnrequested" => Some(SubstrateEvent::Identity(
                IdentityEvent::JudgementUnrequested {
                    extrinsic_index,
                    target_account_id: get_argument_primitive!(&arguments[0], AccountId),
                    registrar_index: get_argument_primitive!(&arguments[1], RegistrarIndex),
                },
            )),
            "SubIdentityAdded" => Some(SubstrateEvent::Identity(IdentityEvent::SubIdentityAdded {
                extrinsic_index,
                sub_account_id: get_argument_primitive!(&arguments[0], AccountId),
                main_account_id: get_argument_primitive!(&arguments[1], AccountId),
                deposit: get_argument_primitive!(&arguments[2], Balance),
            })),
            "SubIdentityRemoved" => Some(SubstrateEvent::Identity(
                IdentityEvent::SubIdentityRemoved {
                    extrinsic_index,
                    sub_account_id: get_argument_primitive!(&arguments[0], AccountId),
                    main_account_id: get_argument_primitive!(&arguments[1], AccountId),
                    freed_deposit: get_argument_primitive!(&arguments[2], Balance),
                },
            )),
            "SubIdentityRevoked" => Some(SubstrateEvent::Identity(
                IdentityEvent::SubIdentityRevoked {
                    extrinsic_index,
                    sub_account_id: get_argument_primitive!(&arguments[0], AccountId),
                    main_account_id: get_argument_primitive!(&arguments[1], AccountId),
                    repatriated_deposit: get_argument_primitive!(&arguments[2], Balance),
                },
            )),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum ImOnlineEvent {
    AllGood {
        extrinsic_index: Option<u32>,
    },
    HeartbeatReceived {
        extrinsic_index: Option<u32>,
        validator_account_id: AccountId,
    },
    SomeOffline {
        identification_tuples: Vec<IdentificationTuple>,
    },
}

impl ImOnlineEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "AllGood" => Some(SubstrateEvent::ImOnline(ImOnlineEvent::AllGood {
                extrinsic_index,
            })),
            "HeartbeatReceived" => {
                Some(SubstrateEvent::ImOnline(ImOnlineEvent::HeartbeatReceived {
                    extrinsic_index,
                    validator_account_id: get_argument_primitive!(&arguments[0], AccountId),
                }))
            }
            "SomeOffline" => Some(SubstrateEvent::ImOnline(ImOnlineEvent::SomeOffline {
                identification_tuples: get_argument_vector!(&arguments[0], IdentificationTuple),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum OffencesEvent {
    Offence {
        extrinsic_index: Option<u32>,
        offence_kind: Kind,
        time_slot: OpaqueTimeSlot,
    },
}

impl OffencesEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "Offence" => Some(SubstrateEvent::Offences(OffencesEvent::Offence {
                extrinsic_index,
                offence_kind: get_argument_primitive!(&arguments[0], OffenceKind),
                time_slot: get_argument_primitive!(&arguments[1], OpaqueTimeSlot),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum ParachainInclusionEvent {
    CandidateBacked {
        extrinsic_index: Option<u32>,
        candidate_receipt: CandidateReceipt,
        head_data: HeadData,
        core_index: CoreIndex,
        group_index: GroupIndex,
    },
    CandidateIncluded {
        extrinsic_index: Option<u32>,
        candidate_receipt: CandidateReceipt,
        head_data: HeadData,
        core_index: CoreIndex,
        group_index: GroupIndex,
    },
    CandidateTimedOut {
        extrinsic_index: Option<u32>,
        candidate_receipt: CandidateReceipt,
        head_data: HeadData,
        core_index: CoreIndex,
    },
}

impl ParachainInclusionEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "CandidateBacked" => Some(SubstrateEvent::ParachainInclusion(Box::new(
                ParachainInclusionEvent::CandidateBacked {
                    extrinsic_index,
                    candidate_receipt: get_argument_primitive!(&arguments[0], CandidateReceipt),
                    head_data: get_argument_primitive!(&arguments[1], ParachainHeadData),
                    core_index: get_argument_primitive!(&arguments[2], CoreIndex),
                    group_index: get_argument_primitive!(&arguments[3], GroupIndex),
                },
            ))),
            "CandidateIncluded" => Some(SubstrateEvent::ParachainInclusion(Box::new(
                ParachainInclusionEvent::CandidateIncluded {
                    extrinsic_index,
                    candidate_receipt: get_argument_primitive!(&arguments[0], CandidateReceipt),
                    head_data: get_argument_primitive!(&arguments[1], ParachainHeadData),
                    core_index: get_argument_primitive!(&arguments[2], CoreIndex),
                    group_index: get_argument_primitive!(&arguments[3], GroupIndex),
                },
            ))),
            "CandidateTimedOut" => Some(SubstrateEvent::ParachainInclusion(Box::new(
                ParachainInclusionEvent::CandidateTimedOut {
                    extrinsic_index,
                    candidate_receipt: get_argument_primitive!(&arguments[0], CandidateReceipt),
                    head_data: get_argument_primitive!(&arguments[1], ParachainHeadData),
                    core_index: get_argument_primitive!(&arguments[2], CoreIndex),
                },
            ))),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum ParachainsEvent {
    CurrentHeadUpdated {
        extrinsic_index: Option<u32>,
        parachain_id: Id,
    },
    CodeUpgradeScheduled {
        extrinsic_index: Option<u32>,
        parachain_id: Id,
    },
    NewHeadNoted {
        extrinsic_index: Option<u32>,
        parachain_id: Id,
    },
    CurrentCodeUpdated {
        extrinsic_index: Option<u32>,
        parachain_id: Id,
    },
    ActionQueued {
        extrinsic_index: Option<u32>,
        parachain_id: Id,
    },
}

impl ParachainsEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "CurrentHeadUpdated" => Some(SubstrateEvent::Parachains(
                ParachainsEvent::CurrentHeadUpdated {
                    extrinsic_index,
                    parachain_id: get_argument_primitive!(&arguments[0], ParachainId),
                },
            )),
            "CodeUpgradeScheduled" => Some(SubstrateEvent::Parachains(
                ParachainsEvent::CodeUpgradeScheduled {
                    extrinsic_index,
                    parachain_id: get_argument_primitive!(&arguments[0], ParachainId),
                },
            )),
            "NewHeadNoted" => Some(SubstrateEvent::Parachains(ParachainsEvent::NewHeadNoted {
                extrinsic_index,
                parachain_id: get_argument_primitive!(&arguments[0], ParachainId),
            })),
            "CurrentCodeUpdated" => Some(SubstrateEvent::Parachains(
                ParachainsEvent::CurrentCodeUpdated {
                    extrinsic_index,
                    parachain_id: get_argument_primitive!(&arguments[0], ParachainId),
                },
            )),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum SessionEvent {
    NewSession {
        extrinsic_index: Option<u32>,
        session_index: SessionIndex,
    },
}

impl SessionEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "NewSession" => Some(SubstrateEvent::Session(SessionEvent::NewSession {
                extrinsic_index,
                session_index: get_argument_primitive!(&arguments[0], SessionIndex),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum StakingEvent {
    Bonded {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        balance: Balance,
    },
    Chilled {
        extrinsic_index: Option<u32>,
        validator_account_id: AccountId,
    },
    EraPaid {
        extrinsic_index: Option<u32>,
        era_index: EraIndex,
        validator_payout: Balance,
        remainder: Balance,
    },
    NominatorKicked {
        extrinsic_index: Option<u32>,
        nominator_account_id: AccountId,
        validator_account_id: AccountId,
    },
    OldSlashingReportDiscarded {
        extrinsic_index: Option<u32>,
        session_index: SessionIndex,
    },
    PayoutStarted {
        extrinsic_index: Option<u32>,
        era_index: EraIndex,
        validator_account_id: AccountId,
    },
    Rewarded {
        extrinsic_index: Option<u32>,
        rewardee_account_id: AccountId,
        amount: Balance,
    },
    Slashed {
        extrinsic_index: Option<u32>,
        validator_account_id: AccountId,
        amount: Balance,
    },
    StakersElected {
        extrinsic_index: Option<u32>,
    },
    StakingElectionFailed {
        extrinsic_index: Option<u32>,
    },
    Unbonded {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        amount: Balance,
    },
    Withdrawn {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        amount: Balance,
    },
}

impl StakingEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "Bonded" => Some(SubstrateEvent::Staking(StakingEvent::Bonded {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                balance: get_argument_primitive!(&arguments[1], Balance),
            })),
            "Chilled" => Some(SubstrateEvent::Staking(StakingEvent::Chilled {
                extrinsic_index,
                validator_account_id: get_argument_primitive!(&arguments[0], AccountId),
            })),
            "EraPaid" | "EraPayout" => Some(SubstrateEvent::Staking(StakingEvent::EraPaid {
                extrinsic_index,
                era_index: get_argument_primitive!(&arguments[0], EraIndex),
                validator_payout: get_argument_primitive!(&arguments[1], Balance),
                remainder: get_argument_primitive!(&arguments[2], Balance),
            })),
            "Kicked" => Some(SubstrateEvent::Staking(StakingEvent::NominatorKicked {
                extrinsic_index,
                nominator_account_id: get_argument_primitive!(&arguments[0], AccountId),
                validator_account_id: get_argument_primitive!(&arguments[1], AccountId),
            })),
            "OldSlashingReportDiscarded" => Some(SubstrateEvent::Staking(
                StakingEvent::OldSlashingReportDiscarded {
                    extrinsic_index,
                    session_index: get_argument_primitive!(&arguments[0], SessionIndex),
                },
            )),
            "PayoutStarted" => Some(SubstrateEvent::Staking(StakingEvent::PayoutStarted {
                extrinsic_index,
                era_index: get_argument_primitive!(&arguments[0], EraIndex),
                validator_account_id: get_argument_primitive!(&arguments[1], AccountId),
            })),
            "Rewarded" | "Reward" => Some(SubstrateEvent::Staking(StakingEvent::Rewarded {
                extrinsic_index,
                rewardee_account_id: get_argument_primitive!(&arguments[0], AccountId),
                amount: get_argument_primitive!(&arguments[1], Balance),
            })),
            "Slashed" | "Slash" => Some(SubstrateEvent::Staking(StakingEvent::Slashed {
                extrinsic_index,
                validator_account_id: get_argument_primitive!(&arguments[0], AccountId),
                amount: get_argument_primitive!(&arguments[1], Balance),
            })),
            "StakersElected" | "StakingElection" => {
                Some(SubstrateEvent::Staking(StakingEvent::StakersElected {
                    extrinsic_index,
                }))
            }
            "StakingElectionFailed" => Some(SubstrateEvent::Staking(
                StakingEvent::StakingElectionFailed { extrinsic_index },
            )),
            "Unbonded" => Some(SubstrateEvent::Staking(StakingEvent::Unbonded {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                amount: get_argument_primitive!(&arguments[1], Balance),
            })),
            "Withdrawn" => Some(SubstrateEvent::Staking(StakingEvent::Withdrawn {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                amount: get_argument_primitive!(&arguments[1], Balance),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum SystemEvent {
    CodeUpdated {
        extrinsic_index: Option<u32>,
    },
    ExtrinsicFailed {
        extrinsic_index: Option<u32>,
        dispatch_error: DispatchError,
        dispatch_info: DispatchInfo,
    },
    ExtrinsicSuccess {
        extrinsic_index: Option<u32>,
        dispatch_info: DispatchInfo,
    },
    KilledAccount {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
    },
    NewAccount {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
    },
}

impl SystemEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "CodeUpdated" => Some(SubstrateEvent::System(SystemEvent::CodeUpdated {
                extrinsic_index,
            })),
            "ExtrinsicSuccess" => Some(SubstrateEvent::System(SystemEvent::ExtrinsicSuccess {
                extrinsic_index,
                dispatch_info: get_argument_primitive!(&arguments[0], DispatchInfo),
            })),
            "ExtrinsicFailed" => Some(SubstrateEvent::System(SystemEvent::ExtrinsicFailed {
                extrinsic_index,
                dispatch_error: get_argument_primitive!(&arguments[0], DispatchError),
                dispatch_info: get_argument_primitive!(&arguments[1], DispatchInfo),
            })),
            "KilledAccount" => Some(SubstrateEvent::System(SystemEvent::KilledAccount {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
            })),
            "NewAccount" => Some(SubstrateEvent::System(SystemEvent::NewAccount {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum UtilityEvent {
    ItemCompleted {
        extrinsic_index: Option<u32>,
    },
    BatchInterrupted {
        extrinsic_index: Option<u32>,
        item_index: u32,
        dispatch_error: DispatchError,
    },
    BatchCompleted {
        extrinsic_index: Option<u32>,
    },
}

impl UtilityEvent {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "ItemCompleted" => Some(SubstrateEvent::Utility(UtilityEvent::ItemCompleted {
                extrinsic_index,
            })),
            "BatchInterrupted" => Some(SubstrateEvent::Utility(UtilityEvent::BatchInterrupted {
                extrinsic_index,
                item_index: get_argument_primitive!(&arguments[0], U32),
                dispatch_error: get_argument_primitive!(&arguments[1], DispatchError),
            })),
            "BatchCompleted" => Some(SubstrateEvent::Utility(UtilityEvent::BatchCompleted {
                extrinsic_index,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum SubstrateEvent {
    Balances(BalancesEvent),
    Identity(IdentityEvent),
    ImOnline(ImOnlineEvent),
    Offences(OffencesEvent),
    ParachainInclusion(Box<ParachainInclusionEvent>),
    Parachains(ParachainsEvent),
    Session(SessionEvent),
    Staking(StakingEvent),
    System(SystemEvent),
    Utility(UtilityEvent),
    Other {
        module_name: String,
        event_name: String,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    },
}

impl SubstrateEvent {
    fn decode_event(
        chain: &Chain,
        metadata: &Metadata,
        bytes: &mut &[u8],
    ) -> Result<Self, DecodeError> {
        let phase = frame_system::Phase::decode(bytes)?;
        let extrinsic_index = match phase {
            frame_system::Phase::ApplyExtrinsic(extrinsic_index) => Some(extrinsic_index),
            _ => None,
        };
        let module_index: u8 = Decode::decode(&mut *bytes)?;
        let event_index: u8 = Decode::decode(&mut *bytes)?;
        let module = if let Some(module) = metadata.modules.get(&module_index) {
            module
        } else {
            return Err(DecodeError::Error(format!(
                "Cannot find module at index {}.",
                module_index
            )));
        };
        let event = if let Some(event) = module.events.get(&event_index) {
            event
        } else {
            return Err(DecodeError::Error(format!(
                "Cannot find event at index {} for module {}.",
                event_index, module.name
            )));
        };
        // decode arguments
        let mut arguments: Vec<Argument> = Vec::new();
        for argument_meta in &event.arguments {
            arguments.push(Argument::decode(
                chain,
                metadata,
                argument_meta,
                &mut *bytes,
            )?);
        }
        // decode topics - unused
        let _topics = Vec::<sp_core::H256>::decode(bytes)?;
        // decode events
        // debug!("Will decode {}.{}.", module.name, event.name);
        let maybe_event = match module.name.as_str() {
            "Balances" => BalancesEvent::from(&event.name, extrinsic_index, arguments.clone())?,
            "Identity" => IdentityEvent::from(&event.name, extrinsic_index, arguments.clone())?,
            "ImOnline" => ImOnlineEvent::from(&event.name, extrinsic_index, arguments.clone())?,
            "Offences" => OffencesEvent::from(&event.name, extrinsic_index, arguments.clone())?,
            "ParaInclusion" => {
                ParachainInclusionEvent::from(&event.name, extrinsic_index, arguments.clone())?
            }
            "Paras" => ParachainsEvent::from(&event.name, extrinsic_index, arguments.clone())?,
            "Session" => SessionEvent::from(&event.name, extrinsic_index, arguments.clone())?,
            "Staking" => StakingEvent::from(&event.name, extrinsic_index, arguments.clone())?,
            "System" => SystemEvent::from(&event.name, extrinsic_index, arguments.clone())?,
            "Utility" => UtilityEvent::from(&event.name, extrinsic_index, arguments.clone())?,
            _ => None,
        };
        let substrate_event = if let Some(substrate_event) = maybe_event {
            debug!("Decoded event {}.{}.", module.name, event.name);
            substrate_event
        } else {
            debug!(
                "Decoded non-specified event {}.{}.",
                module.name, event.name
            );
            SubstrateEvent::Other {
                module_name: module.name.clone(),
                event_name: event.name.clone(),
                extrinsic_index,
                arguments,
            }
        };
        Ok(substrate_event)
    }

    pub fn decode_events(
        chain: &Chain,
        metadata: &Metadata,
        block: Block,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Vec<Self>> {
        let event_count = <Compact<u32>>::decode(bytes)?.0;
        let mut events: Vec<Self> = Vec::with_capacity(event_count as usize);
        for event_index in 0..event_count {
            match SubstrateEvent::decode_event(chain, metadata, &mut *bytes) {
                Ok(event) => events.push(event),
                Err(error) => error!(
                    "Error decoding event #{} for block #{}: {:?}",
                    event_index,
                    block.header.get_number().unwrap(),
                    error
                ),
            }
        }
        Ok(events)
    }
}
