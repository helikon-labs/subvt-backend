use crate::metadata::{Metadata, EventArg};
use frame_support::{
    dispatch::{DispatchInfo, DispatchError, DispatchResult},
    traits::BalanceStatus,
    weights::Weight,
};
use log::{debug};
use pallet_bounties::BountyIndex;
use pallet_collective::{MemberCount, ProposalIndex};
use pallet_democracy::{PropIndex, ReferendumIndex, VoteThreshold};
use pallet_election_provider_multi_phase::ElectionCompute;
use pallet_gilt::ActiveIndex;
use pallet_identity::RegistrarIndex;
use pallet_multisig::Timepoint;
use pallet_scheduler::TaskAddress;
use pallet_staking::EraIndex;
use parity_scale_codec::{Compact, Decode, Input};
use polkadot_core_primitives::{AccountIndex, Hash};
use polkadot_primitives::v1::{
    Balance, BlockNumber, CandidateReceipt, CoreIndex, GroupIndex, HeadData, HrmpChannelId, Id,
};
use polkadot_runtime::ProxyType;
use polkadot_runtime_common::{auctions::AuctionIndex, claims::EthereumAddress};
use polkadot_runtime_parachains::ump::MessageId;
use sp_authority_discovery::AuthorityId;
use sp_finality_grandpa::AuthorityList;
use sp_staking::{offence::Kind, SessionIndex};
use subvt_types::crypto::AccountId;

pub enum SubstrateEvent {
    ExtrinsicSuccess(u32),
    ExtrinsicFailed(u32),
}

type CallHash = [u8; 32];
type OpaqueTimeSlot = Vec<u8>;

impl SubstrateEvent {
    fn decode_event_arg(
        events_bytes: &mut &[u8],
        event_arg: &EventArg,
    ) {
        match event_arg {
            EventArg::Vec(event_arg) => {
                debug!("|-- Event arg is vector.");
                let len: Compact<u32> = Decode::decode(events_bytes).unwrap();
                for _ in 0..len.0 {
                    SubstrateEvent::decode_event_arg(events_bytes, event_arg);
                }
            }
            EventArg::Option(event_arg) => {
                debug!("|-- Event arg is option.");
                match events_bytes.read_byte().unwrap() {
                    0 => (),
                    1 => {
                        SubstrateEvent::decode_event_arg(events_bytes, event_arg);
                    }
                    _ => {
                        panic!("unexpected first byte decoding Option");
                    }
                }
            }
            EventArg::Tuple(event_args) => {
                debug!("|-- Event arg is tuple.");
                for event_arg in event_args {
                    SubstrateEvent::decode_event_arg(events_bytes, event_arg);
                }
            }
            EventArg::Primitive(name) => {
                if name == "DispatchInfo" {
                    let _: DispatchInfo = Decode::decode(events_bytes).unwrap();
                } else if name == "DispatchError" {
                    let _: DispatchError = Decode::decode(events_bytes).unwrap();
                } else if name == "DispatchResult" {
                    let _: DispatchResult = Decode::decode(events_bytes).unwrap();
                } else if name == "CandidateReceipt<Hash>" || name == "CandidateReceipt<T::Hash>" {
                    let _: CandidateReceipt = Decode::decode(events_bytes).unwrap();
                } else if name == "HeadData" {
                    let _: HeadData = Decode::decode(events_bytes).unwrap();
                } else if name == "CoreIndex" {
                    let _: CoreIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "GroupIndex" {
                    let _: GroupIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "AuthorityId" {
                    let _: AuthorityId = Decode::decode(events_bytes).unwrap();
                } else if name == "AccountId" {
                    let _: AccountId = Decode::decode(events_bytes).unwrap();
                } else if name == "u8" {
                    let _: u8 = Decode::decode(events_bytes).unwrap();
                } else if name == "u16" {
                    let _: u16 = Decode::decode(events_bytes).unwrap();
                } else if name == "u32" {
                    let _: u32 = Decode::decode(events_bytes).unwrap();
                } else if name == "bool" {
                    let _: bool = Decode::decode(events_bytes).unwrap();
                } else if name == "ParaId" {
                    let _: Id = Decode::decode(events_bytes).unwrap();
                } else if name == "Balance" {
                    let _: Balance = Decode::decode(events_bytes).unwrap();
                } else if name == "Status" {
                    let _: BalanceStatus = Decode::decode(events_bytes).unwrap();
                } else if name == "BlockNumber" || name == "LeasePeriod" {
                    let _: BlockNumber = Decode::decode(events_bytes).unwrap();
                } else if name == "EraIndex" {
                    let _: EraIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "SessionIndex" {
                    let _: SessionIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "PropIndex" {
                    let _: PropIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "BountyIndex" {
                    let _: BountyIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "AuctionIndex" {
                    let _: AuctionIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "AccountIndex" {
                    let _: AccountIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "xcm::v0::Outcome" {
                    let _: xcm::v0::Outcome = Decode::decode(events_bytes).unwrap();
                } else if name == "Outcome" {
                    let _: xcm::latest::Outcome = Decode::decode(events_bytes).unwrap();
                } else if name == "Hash" {
                    let _: Hash = Decode::decode(events_bytes).unwrap();
                } else if name == "Xcm<()>" {
                    let _: xcm::latest::Xcm<()> = Decode::decode(events_bytes).unwrap();
                } else if name == "HrmpChannelId" {
                    let _: HrmpChannelId = Decode::decode(events_bytes).unwrap();
                } else if name == "ProxyType" {
                    let _: ProxyType = Decode::decode(events_bytes).unwrap();
                } else if name == "MemberCount" {
                    let _: MemberCount = Decode::decode(events_bytes).unwrap();
                } else if name == "MessageId" {
                    let _: MessageId = Decode::decode(events_bytes).unwrap();
                } else if name == "MultiLocation" {
                    let _: xcm::latest::MultiLocation = Decode::decode(events_bytes).unwrap();
                } else if name == "CallHash" {
                    let _: CallHash = Decode::decode(events_bytes).unwrap();
                } else if name == "ActiveIndex" {
                    let _: ActiveIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "ProposalIndex" {
                    let _: ProposalIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "ReferendumIndex" {
                    let _: ReferendumIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "RegistrarIndex" {
                    let _: RegistrarIndex = Decode::decode(events_bytes).unwrap();
                } else if name == "EthereumAddress" {
                    let _: EthereumAddress = Decode::decode(events_bytes).unwrap();
                } else if name == "VoteThreshold" {
                    let _: VoteThreshold = Decode::decode(events_bytes).unwrap();
                } else if name == "ElectionCompute" {
                    let _: ElectionCompute = Decode::decode(events_bytes).unwrap();
                } else if name == "Timepoint<BlockNumber>" {
                    let _: Timepoint<BlockNumber> = Decode::decode(events_bytes).unwrap();
                } else if name == "T::BlockNumber" {
                    let _: BlockNumber = Decode::decode(events_bytes).unwrap();
                } else if name == "Weight" {
                    let _: Weight = Decode::decode(events_bytes).unwrap();
                } else if name == "Kind" {
                    let _: Kind = Decode::decode(events_bytes).unwrap();
                } else if name == "TaskAddress<BlockNumber>" {
                    let _: TaskAddress<BlockNumber> = Decode::decode(events_bytes).unwrap();
                } else if name == "AuthorityList" {
                    let _: AuthorityList = Decode::decode(events_bytes).unwrap();
                } else if name == "OpaqueTimeSlot" {
                    let _: OpaqueTimeSlot = Decode::decode(events_bytes).unwrap();
                } else if name == "sp_std::marker::PhantomData<(AccountId, Event)>" {
                    panic!("TechnicalMembership.Dummy(sp_std::marker::PhantomData<(AccountId, Event)>, ) - should never be used.")
                } else {
                    panic!("|-- Event arg is unknown primitive [{}].", name);
                }
                debug!("|-- Decoded {}.", name);
            }
        }
    }

    fn decode_event(metadata: &Metadata, bytes: &mut &[u8]) -> anyhow::Result<Self> {
        let phase = frame_system::Phase::decode(bytes).unwrap();
        let module_index = bytes.read_byte()?;
        let event_index = bytes.read_byte()?;
        let module = metadata.modules.get(&module_index).unwrap();
        let event = module.events.get(&event_index).unwrap();
        debug!("{}.{}.", module.name, event.name);
        // decode arguments
        for argument in &event.arguments {
            SubstrateEvent::decode_event_arg(bytes, argument);
        }
        // decode topics - unsued
        let _topics = Vec::<sp_core::H256>::decode(bytes)?;
        // check extrinsic success / failed
        if module.name == "System" {
            if event.name == "ExtrinsicSuccess" {
                if let frame_system::Phase::ApplyExtrinsic(extrinsic_index) = phase {
                    return Ok(SubstrateEvent::ExtrinsicSuccess(extrinsic_index));
                }
            } else if event.name == "ExtrinsicFailed" {
                if let frame_system::Phase::ApplyExtrinsic(extrinsic_index) = phase {
                    return Ok(SubstrateEvent::ExtrinsicFailed(extrinsic_index));
                }
            }
        }
        Ok(SubstrateEvent::ExtrinsicFailed(3))
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