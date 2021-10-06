use crate::substrate::OpaqueTimeSlot;
use crate::{
    crypto::AccountId,
    substrate::{
        argument::{
            get_argument_primitive, get_argument_vector, Argument, ArgumentPrimitive,
            IdentificationTuple,
        },
        error::DecodeError,
        metadata::Metadata,
        Balance,
    },
};
use frame_support::dispatch::{DispatchError, DispatchInfo};
use log::{debug, warn};
use pallet_identity::RegistrarIndex;
use pallet_staking::EraIndex;
use parity_scale_codec::{Compact, Decode, Input};
use polkadot_primitives::v1::{CandidateReceipt, CoreIndex, GroupIndex, HeadData, Id};
use sp_authority_discovery::AuthorityId;
use sp_staking::offence::Kind;
use sp_staking::SessionIndex;

#[derive(Debug)]
pub enum Balances {
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

impl Balances {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "BalanceSet" => Some(SubstrateEvent::Balances(Balances::BalanceSet {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                free_amount: get_argument_primitive!(&arguments[1], Balance),
                reserved_amount: get_argument_primitive!(&arguments[2], Balance),
            })),
            "Deposit" => Some(SubstrateEvent::Balances(Balances::Deposit {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                amount: get_argument_primitive!(&arguments[1], Balance),
            })),
            "Transfer" => Some(SubstrateEvent::Balances(Balances::Transfer {
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
pub enum Identity {
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

impl Identity {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "IdentityCleared" => Some(SubstrateEvent::Identity(Identity::IdentityCleared {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                returned_balance: get_argument_primitive!(&arguments[1], Balance),
            })),
            "IdentityKilled" => Some(SubstrateEvent::Identity(Identity::IdentityKilled {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                slashed_balance: get_argument_primitive!(&arguments[1], Balance),
            })),
            "IdentitySet" => Some(SubstrateEvent::Identity(Identity::IdentitySet {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
            })),
            "JudgementGiven" => Some(SubstrateEvent::Identity(Identity::JudgementGiven {
                extrinsic_index,
                target_account_id: get_argument_primitive!(&arguments[0], AccountId),
                registrar_index: get_argument_primitive!(&arguments[1], RegistrarIndex),
            })),
            "JudgementRequested" => Some(SubstrateEvent::Identity(Identity::JudgementRequested {
                extrinsic_index,
                target_account_id: get_argument_primitive!(&arguments[0], AccountId),
                registrar_index: get_argument_primitive!(&arguments[1], RegistrarIndex),
            })),
            "JudgementUnrequested" => {
                Some(SubstrateEvent::Identity(Identity::JudgementUnrequested {
                    extrinsic_index,
                    target_account_id: get_argument_primitive!(&arguments[0], AccountId),
                    registrar_index: get_argument_primitive!(&arguments[1], RegistrarIndex),
                }))
            }
            "SubIdentityAdded" => Some(SubstrateEvent::Identity(Identity::SubIdentityAdded {
                extrinsic_index,
                sub_account_id: get_argument_primitive!(&arguments[0], AccountId),
                main_account_id: get_argument_primitive!(&arguments[1], AccountId),
                deposit: get_argument_primitive!(&arguments[2], Balance),
            })),
            "SubIdentityRemoved" => Some(SubstrateEvent::Identity(Identity::SubIdentityRemoved {
                extrinsic_index,
                sub_account_id: get_argument_primitive!(&arguments[0], AccountId),
                main_account_id: get_argument_primitive!(&arguments[1], AccountId),
                freed_deposit: get_argument_primitive!(&arguments[2], Balance),
            })),
            "SubIdentityRevoked" => Some(SubstrateEvent::Identity(Identity::SubIdentityRevoked {
                extrinsic_index,
                sub_account_id: get_argument_primitive!(&arguments[0], AccountId),
                main_account_id: get_argument_primitive!(&arguments[1], AccountId),
                repatriated_deposit: get_argument_primitive!(&arguments[2], Balance),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum ImOnline {
    AllGood {
        extrinsic_index: Option<u32>,
    },
    HeartbeatReceived {
        extrinsic_index: Option<u32>,
        authority_id: AuthorityId,
    },
    SomeOffline {
        extrinsic_index: Option<u32>,
        identification_tuples: Vec<IdentificationTuple>,
    },
}

impl ImOnline {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "AllGood" => Some(SubstrateEvent::ImOnline(ImOnline::AllGood {
                extrinsic_index,
            })),
            "HeartbeatReceived" => Some(SubstrateEvent::ImOnline(ImOnline::HeartbeatReceived {
                extrinsic_index,
                authority_id: get_argument_primitive!(&arguments[0], AuthorityId),
            })),
            "SomeOffline" => Some(SubstrateEvent::ImOnline(ImOnline::SomeOffline {
                extrinsic_index,
                identification_tuples: get_argument_vector!(&arguments[0], IdentificationTuple),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum Offences {
    Offence {
        extrinsic_index: Option<u32>,
        offence_kind: Kind,
        time_slot: OpaqueTimeSlot,
    },
}

impl Offences {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "Offence" => Some(SubstrateEvent::Offences(Offences::Offence {
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
pub enum ParaInclusion {
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

impl ParaInclusion {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "CandidateBacked" => Some(SubstrateEvent::ParaInclusion(Box::new(
                ParaInclusion::CandidateBacked {
                    extrinsic_index,
                    candidate_receipt: get_argument_primitive!(&arguments[0], CandidateReceipt),
                    head_data: get_argument_primitive!(&arguments[1], ParachainHeadData),
                    core_index: get_argument_primitive!(&arguments[2], CoreIndex),
                    group_index: get_argument_primitive!(&arguments[3], GroupIndex),
                },
            ))),
            "CandidateIncluded" => Some(SubstrateEvent::ParaInclusion(Box::new(
                ParaInclusion::CandidateIncluded {
                    extrinsic_index,
                    candidate_receipt: get_argument_primitive!(&arguments[0], CandidateReceipt),
                    head_data: get_argument_primitive!(&arguments[1], ParachainHeadData),
                    core_index: get_argument_primitive!(&arguments[2], CoreIndex),
                    group_index: get_argument_primitive!(&arguments[3], GroupIndex),
                },
            ))),
            "CandidateTimedOut" => Some(SubstrateEvent::ParaInclusion(Box::new(
                ParaInclusion::CandidateTimedOut {
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
pub enum Paras {
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

impl Paras {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "CurrentHeadUpdated" => Some(SubstrateEvent::Paras(Paras::CurrentHeadUpdated {
                extrinsic_index,
                parachain_id: get_argument_primitive!(&arguments[0], ParachainId),
            })),
            "CodeUpgradeScheduled" => Some(SubstrateEvent::Paras(Paras::CodeUpgradeScheduled {
                extrinsic_index,
                parachain_id: get_argument_primitive!(&arguments[0], ParachainId),
            })),
            "NewHeadNoted" => Some(SubstrateEvent::Paras(Paras::NewHeadNoted {
                extrinsic_index,
                parachain_id: get_argument_primitive!(&arguments[0], ParachainId),
            })),
            "CurrentCodeUpdated" => Some(SubstrateEvent::Paras(Paras::CurrentCodeUpdated {
                extrinsic_index,
                parachain_id: get_argument_primitive!(&arguments[0], ParachainId),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum Session {
    NewSession {
        extrinsic_index: Option<u32>,
        session_index: SessionIndex,
    },
}

impl Session {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "NewSession" => Some(SubstrateEvent::Session(Session::NewSession {
                extrinsic_index,
                session_index: get_argument_primitive!(&arguments[0], SessionIndex),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum Staking {
    Bonded {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        balance: Balance,
    },
    Chilled {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
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
        nominator_account_id: AccountId,
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

impl Staking {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "Bonded" => Some(SubstrateEvent::Staking(Staking::Bonded {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                balance: get_argument_primitive!(&arguments[1], Balance),
            })),
            "Chilled" => Some(SubstrateEvent::Staking(Staking::Chilled {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
            })),
            "EraPaid" | "EraPayout" => Some(SubstrateEvent::Staking(Staking::EraPaid {
                extrinsic_index,
                era_index: get_argument_primitive!(&arguments[0], EraIndex),
                validator_payout: get_argument_primitive!(&arguments[1], Balance),
                remainder: get_argument_primitive!(&arguments[2], Balance),
            })),
            "Kicked" => Some(SubstrateEvent::Staking(Staking::NominatorKicked {
                extrinsic_index,
                nominator_account_id: get_argument_primitive!(&arguments[0], AccountId),
                validator_account_id: get_argument_primitive!(&arguments[1], AccountId),
            })),
            "OldSlashingReportDiscarded" => Some(SubstrateEvent::Staking(
                Staking::OldSlashingReportDiscarded {
                    extrinsic_index,
                    session_index: get_argument_primitive!(&arguments[0], SessionIndex),
                },
            )),
            "PayoutStarted" => Some(SubstrateEvent::Staking(Staking::PayoutStarted {
                extrinsic_index,
                era_index: get_argument_primitive!(&arguments[0], EraIndex),
                validator_account_id: get_argument_primitive!(&arguments[1], AccountId),
            })),
            "Rewarded" | "Reward" => Some(SubstrateEvent::Staking(Staking::Rewarded {
                extrinsic_index,
                nominator_account_id: get_argument_primitive!(&arguments[0], AccountId),
                amount: get_argument_primitive!(&arguments[1], Balance),
            })),
            "Slashed" | "Slash" => Some(SubstrateEvent::Staking(Staking::Slashed {
                extrinsic_index,
                validator_account_id: get_argument_primitive!(&arguments[0], AccountId),
                amount: get_argument_primitive!(&arguments[1], Balance),
            })),
            "StakersElected" | "StakingElection" => {
                Some(SubstrateEvent::Staking(Staking::StakersElected {
                    extrinsic_index,
                }))
            }
            "StakingElectionFailed" => {
                Some(SubstrateEvent::Staking(Staking::StakingElectionFailed {
                    extrinsic_index,
                }))
            }
            "Unbonded" => Some(SubstrateEvent::Staking(Staking::Unbonded {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
                amount: get_argument_primitive!(&arguments[1], Balance),
            })),
            "Withdrawn" => Some(SubstrateEvent::Staking(Staking::Withdrawn {
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
pub enum System {
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

impl System {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "CodeUpdated" => Some(SubstrateEvent::System(System::CodeUpdated {
                extrinsic_index,
            })),
            "ExtrinsicSuccess" => Some(SubstrateEvent::System(System::ExtrinsicSuccess {
                extrinsic_index,
                dispatch_info: get_argument_primitive!(&arguments[0], DispatchInfo),
            })),
            "ExtrinsicFailed" => Some(SubstrateEvent::System(System::ExtrinsicFailed {
                extrinsic_index,
                dispatch_error: get_argument_primitive!(&arguments[0], DispatchError),
                dispatch_info: get_argument_primitive!(&arguments[1], DispatchInfo),
            })),
            "KilledAccount" => Some(SubstrateEvent::System(System::KilledAccount {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
            })),
            "NewAccount" => Some(SubstrateEvent::System(System::NewAccount {
                extrinsic_index,
                account_id: get_argument_primitive!(&arguments[0], AccountId),
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum Utility {
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

impl Utility {
    pub fn from(
        name: &str,
        extrinsic_index: Option<u32>,
        arguments: Vec<Argument>,
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            "ItemCompleted" => Some(SubstrateEvent::Utility(Utility::ItemCompleted {
                extrinsic_index,
            })),
            "BatchInterrupted" => Some(SubstrateEvent::Utility(Utility::BatchInterrupted {
                extrinsic_index,
                item_index: get_argument_primitive!(&arguments[0], U32),
                dispatch_error: get_argument_primitive!(&arguments[1], DispatchError),
            })),
            "BatchCompleted" => Some(SubstrateEvent::Utility(Utility::BatchCompleted {
                extrinsic_index,
            })),
            _ => None,
        };
        Ok(maybe_event)
    }
}

#[derive(Debug)]
pub enum SubstrateEvent {
    Balances(Balances),
    Identity(Identity),
    ImOnline(ImOnline),
    Offences(Offences),
    ParaInclusion(Box<ParaInclusion>),
    Paras(Paras),
    Session(Session),
    Staking(Staking),
    System(System),
    Utility(Utility),
    Other {
        module_name: String,
        event_name: String,
        arguments: Vec<Argument>,
    },
}

impl SubstrateEvent {
    fn decode_event(metadata: &Metadata, bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        let phase = frame_system::Phase::decode(bytes)?;
        let extrinsic_index = match phase {
            frame_system::Phase::ApplyExtrinsic(extrinsic_index) => Some(extrinsic_index),
            _ => None,
        };
        let module_index = bytes.read_byte()?;
        let event_index = bytes.read_byte()?;
        let module = metadata.modules.get(&module_index).unwrap();
        let event = module.events.get(&event_index).unwrap();
        // decode arguments
        let mut arguments: Vec<Argument> = Vec::new();
        for argument_meta in &event.arguments {
            arguments.push(Argument::decode(argument_meta, &mut *bytes).unwrap());
        }
        // decode topics - unused
        let _topics = Vec::<sp_core::H256>::decode(bytes)?;
        // decode events
        // debug!("Will decode {}.{}.", module.name, event.name);
        let maybe_event = match module.name.as_str() {
            "Balances" => Balances::from(&event.name, extrinsic_index, arguments.clone())?,
            "Identity" => Identity::from(&event.name, extrinsic_index, arguments.clone())?,
            "ImOnline" => ImOnline::from(&event.name, extrinsic_index, arguments.clone())?,
            "Offences" => Offences::from(&event.name, extrinsic_index, arguments.clone())?,
            "ParaInclusion" => {
                ParaInclusion::from(&event.name, extrinsic_index, arguments.clone())?
            }
            "Paras" => Paras::from(&event.name, extrinsic_index, arguments.clone())?,
            "Session" => Session::from(&event.name, extrinsic_index, arguments.clone())?,
            "Staking" => Staking::from(&event.name, extrinsic_index, arguments.clone())?,
            "System" => System::from(&event.name, extrinsic_index, arguments.clone())?,
            "Utility" => Utility::from(&event.name, extrinsic_index, arguments.clone())?,
            _ => None,
        };
        let substrate_event = if let Some(substrate_event) = maybe_event {
            debug!("Decoded event {}.{}.", module.name, event.name);
            substrate_event
        } else {
            warn!(
                "Decoded non-specified event {}.{}.",
                module.name, event.name
            );
            SubstrateEvent::Other {
                module_name: module.name.clone(),
                event_name: event.name.clone(),
                arguments,
            }
        };
        Ok(substrate_event)
    }

    pub fn decode_events(metadata: &Metadata, bytes: &mut &[u8]) -> anyhow::Result<Vec<Self>> {
        let event_count = <Compact<u32>>::decode(bytes)?.0;
        let mut events: Vec<Self> = Vec::with_capacity(event_count as usize);
        for _ in 0..event_count {
            events.push(SubstrateEvent::decode_event(metadata, &mut *bytes)?);
        }
        Ok(events)
    }
}
