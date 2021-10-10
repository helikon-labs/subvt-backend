use crate::{
    crypto::AccountId,
    substrate::{
        error::DecodeError,
        metadata::{ArgumentMeta, Metadata},
        CallHash, Chain, MultiAddress, OpaqueTimeSlot, RewardDestination, SlotRange,
    },
};
use frame_support::{
    dispatch::{DispatchError, DispatchInfo, DispatchResult},
    traits::BalanceStatus,
    weights::Weight,
};
use pallet_bounties::BountyIndex;
use pallet_collective::{MemberCount, ProposalIndex};
use pallet_democracy::{PropIndex, ReferendumIndex, VoteThreshold};
use pallet_election_provider_multi_phase::ElectionCompute;
use pallet_gilt::ActiveIndex;
use pallet_identity::RegistrarIndex;
use pallet_multisig::Timepoint;
use pallet_scheduler::TaskAddress;
use pallet_staking::{EraIndex, Exposure};
use parity_scale_codec::{Compact, Decode, Input};
use polkadot_core_primitives::{AccountIndex, Hash};
use polkadot_primitives::v1::{
    Balance, BlockNumber, CandidateReceipt, CoreIndex, GroupIndex, HeadData, HrmpChannelId, Id,
};
use polkadot_runtime::ProxyType;
use polkadot_runtime_common::{auctions::AuctionIndex, claims::EthereumAddress};
use polkadot_runtime_parachains::ump::MessageId;
use sp_finality_grandpa::AuthorityList;
use sp_staking::{offence::Kind, SessionIndex};

pub type IdentificationTuple = (AccountId, Exposure<AccountId, Balance>);

#[derive(Clone, Debug)]
pub enum Argument {
    Option(Box<Option<Argument>>),
    Primitive(Box<ArgumentPrimitive>),
    Tuple(Vec<Argument>),
    Vec(Vec<Argument>),
}

#[derive(Clone, Debug)]
pub enum ArgumentPrimitive {
    AccountId(AccountId),
    AccountIndex(AccountIndex),
    AuctionIndex(AuctionIndex),
    Balance(Balance),
    BalanceStatus(BalanceStatus),
    Bool(bool),
    BountyIndex(BountyIndex),
    BlockNumber(BlockNumber),
    CallHash(CallHash),
    CompactBalance(Compact<Balance>),
    CompactEraIndex(Compact<EraIndex>),
    CompactU32(Compact<u32>),
    CandidateReceipt(CandidateReceipt),
    CollectiveMemberCount(MemberCount),
    CollectiveProposalIndex(ProposalIndex),
    CoreIndex(CoreIndex),
    DemocracyProposalIndex(PropIndex),
    DemocracyVoteThreshold(VoteThreshold),
    DispatchError(DispatchError),
    DispatchInfo(DispatchInfo),
    DispatchResult(DispatchResult),
    ElectionCompute(ElectionCompute),
    EraIndex(EraIndex),
    EthereumAddress(EthereumAddress),
    GiltActiveIndex(ActiveIndex),
    GrandpaAuthorityList(AuthorityList),
    GroupIndex(GroupIndex),
    Hash(Hash),
    IdentificationTuple(IdentificationTuple),
    Moment(Compact<u64>),
    MultiAddress(MultiAddress),
    MultiLocation(xcm::latest::MultiLocation),
    MultisigTimepoint(Timepoint<BlockNumber>),
    OffenceKind(Kind),
    OpaqueTimeSlot(OpaqueTimeSlot),
    ParachainHeadData(HeadData),
    ParachainHRMPChannelId(HrmpChannelId),
    ParachainId(Id),
    ParachainLeasePeriod(BlockNumber),
    ParachainUMPMessageId(MessageId),
    RewardDestination(RewardDestination),
    U8(u8),
    U16(u16),
    U32(u32),
    _PhantomData,
    ProxyType(ProxyType),
    ReferendumIndex(ReferendumIndex),
    RegistrarIndex(RegistrarIndex),
    SchedulerTaskAddress(TaskAddress<BlockNumber>),
    SessionIndex(SessionIndex),
    SlotRange(SlotRange),
    Weight(Weight),
    Xcm(xcm::latest::Xcm<()>),
    XcmOutcome(Box<xcm::latest::Outcome>),
    XcmV0Outcome(Box<xcm::v0::Outcome>),
}

pub fn extract_argument_primitive(argument: &Argument) -> Result<ArgumentPrimitive, DecodeError> {
    match argument {
        Argument::Primitive(argument_primitive) => Ok(*argument_primitive.clone()),
        _ => Err(DecodeError::Error(format!(
            "Cannot extract argument primitive: {:?}",
            argument
        ))),
    }
}

macro_rules! get_argument_primitive {
    ($argument_expr: expr, $argument_primitive_type: ident) => {{
        let argument_primitive =
            crate::substrate::argument::extract_argument_primitive($argument_expr)?;
        match argument_primitive {
            ArgumentPrimitive::$argument_primitive_type(primitive) => Ok(primitive),
            _ => Err(DecodeError::Error(format!(
                "Cannot get argument primitive {:?}.",
                argument_primitive
            ))),
        }?
    }};
}
pub(crate) use get_argument_primitive;

macro_rules! get_argument_vector {
    ($argument_expr: expr, $argument_primitive_type: ident) => {{
        let argument_vector = match $argument_expr {
            Argument::Vec(argument_vector) => Ok(&*argument_vector),
            _ => Err(DecodeError::Error(
                "Cannot get argument vector.".to_string(),
            )),
        }?;
        let mut result_vector = Vec::new();
        for argument in argument_vector {
            let argument_primitive =
                crate::substrate::argument::extract_argument_primitive(argument)?;
            let element = match argument_primitive {
                ArgumentPrimitive::$argument_primitive_type(element) => Ok(element),
                _ => Err(DecodeError::Error(format!(
                    "Cannot get argument primitive {:?}.",
                    argument_primitive
                ))),
            }?;
            result_vector.push(element);
        }
        result_vector
    }};
}
pub(crate) use get_argument_vector;

macro_rules! generate_argument_primitive_decoder_impl {
    ([$(($name: literal, $decode_function_name: ident, $argument_primitive_enum_case_name: ident),)+]) => {
        impl ArgumentPrimitive {
            $(
                pub fn $decode_function_name(bytes: &mut &[u8]) -> Result<ArgumentPrimitive, ArgumentDecodeError> {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => Ok(ArgumentPrimitive::$argument_primitive_enum_case_name(decoded)),
                        Err(_) => Err(ArgumentDecodeError::DecodeError(format!("Cannot decode {}.", $name))),
                    }
                }
            )+
            pub fn decode(name: &str, bytes: &mut &[u8]) -> Result<ArgumentPrimitive, ArgumentDecodeError> {
                match name {
                    $(
                        $name => ArgumentPrimitive::$decode_function_name(&mut *bytes),
                    )+
                    _ => Err(ArgumentDecodeError::UnknownPrimitiveType(name.to_string()))
                }
            }
        }
    };
}

generate_argument_primitive_decoder_impl! {[
    ("AccountId", decode_account_id, AccountId),
    ("T::AccountId", decode_account_id_t, AccountId),
    ("AccountIndex", decode_account_index, AccountIndex),
    ("AuctionIndex", decode_auction_index, AuctionIndex),
    ("AuthorityId", decode_authority_id, AccountId),
    ("Balance", decode_balance, Balance),
    ("BalanceOf<T>", decode_balance_of_t, Balance),
    ("Status", decode_balance_status, BalanceStatus),
    ("bool", decode_bool, Bool),
    ("BountyIndex", decode_bounty_index, BountyIndex),
    ("BlockNumber", decode_block_number, BlockNumber),
    ("T::BlockNumber", decode_t_block_number, BlockNumber),
    ("CallHash", decode_call_hash, CallHash),
    ("CandidateReceipt<Hash>", decode_candidate_receipt, CandidateReceipt),
    ("CandidateReceipt<T::Hash>", decode_candidate_receipt_t, CandidateReceipt),
    ("MemberCount", decode_collective_member_count, CollectiveMemberCount),
    ("ProposalIndex", decode_collective_proposal_index, CollectiveProposalIndex),
    ("Compact<BalanceOf<T>>", decode_compact_balance, CompactBalance),
    ("CompactEraIndex", decode_compact_era_index, CompactEraIndex),
    ("Compact<u32>", decode_compact_u32, CompactU32),
    ("CoreIndex", decode_core_index, CoreIndex),
    ("PropIndex", decode_democracy_proposal_index, DemocracyProposalIndex),
    ("VoteThreshold", decode_democracy_vote_threshold, DemocracyVoteThreshold),
    ("DispatchInfo", decode_dispatch_info, DispatchInfo),
    ("DispatchError", decode_dispatch_error, DispatchError),
    ("DispatchResult", decode_dispatch_result, DispatchResult),
    ("ElectionCompute", decode_election_compute, ElectionCompute),
    ("EraIndex", decode_era_index, EraIndex),
    ("EthereumAddress", decode_ethereum_address, EthereumAddress),
    ("ActiveIndex", decode_gilt_active_index, GiltActiveIndex),
    ("AuthorityList", decode_granpa_authority_list, GrandpaAuthorityList),
    ("GroupIndex", decode_group_index, GroupIndex),
    ("Hash", decode_hash, Hash),
    ("IdentificationTuple", decode_identification_tuple, IdentificationTuple),
    ("MultiLocation", decode_multi_location, MultiLocation),
    ("Timepoint<BlockNumber>", decode_multisig_timepoint, MultisigTimepoint),
    ("Kind", decode_offence_kind, OffenceKind),
    ("OpaqueTimeSlot", decode_opaque_time_slot, OpaqueTimeSlot),
    ("HeadData", decode_parachain_head_data, ParachainHeadData),
    ("HrmpChannelId", decode_parachain_hrmp_channel_id, ParachainHRMPChannelId),
    ("ParaId", decode_parachain_id, ParachainId),
    ("LeasePeriod", decode_parachain_lease_period, ParachainLeasePeriod),
    ("MessageId", decode_parachain_ump_message_id, ParachainUMPMessageId),
    ("Compact<T::Moment>", decode_moment, Moment),
    ("u8", decode_u8, U8),
    ("u16", decode_u16, U16),
    ("u32", decode_u32, U32),
    ("ProxyType", decode_proxy_type, ProxyType),
    ("ReferendumIndex", decode_referendum_index, ReferendumIndex),
    ("RegistrarIndex", decode_registrar_index, RegistrarIndex),
    ("RewardDestination<T::AccountId>", decode_reward_destination, RewardDestination),
    ("TaskAddress<BlockNumber>", decode_scheduler_task_address, SchedulerTaskAddress),
    ("SessionIndex", decode_session_index, SessionIndex),
    ("SlotRange", decode_slot_range, SlotRange),
    ("Weight", decode_weight, Weight),
    ("Xcm<()>", decode_xcm, Xcm),
    ("Outcome", decode_xcm_outcome, XcmOutcome),
    ("xcm::v0::Outcome", decode_xcm_v0_outcome, XcmV0Outcome),
]}

#[derive(thiserror::Error, Clone, Debug)]
pub enum ArgumentDecodeError {
    #[error("Decode error: {0}")]
    DecodeError(String),
    #[error("Unknown primitive type: {0}")]
    UnknownPrimitiveType(String),
    #[error("Unsupported primitive type: {0}")]
    UnsupportedPrimitiveType(String),
}

impl Argument {
    pub fn decode(
        chain: &Chain,
        metadata: &Metadata,
        argument_meta: &ArgumentMeta,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Self, ArgumentDecodeError> {
        use ArgumentDecodeError::*;

        match argument_meta {
            ArgumentMeta::Vec(argument_meta) => {
                let length: Compact<u32> = match Decode::decode(bytes) {
                    Ok(length) => length,
                    Err(_) => {
                        return Err(DecodeError(
                            "Cannot decode length for vector argument.".to_string(),
                        ));
                    }
                };
                let mut result: Vec<Argument> = Vec::new();
                for _ in 0..length.0 {
                    result.push(Argument::decode(
                        chain,
                        metadata,
                        argument_meta.as_ref(),
                        &mut *bytes,
                    )?);
                }
                Ok(Argument::Vec(result))
            }
            ArgumentMeta::Option(argument_meta) => match bytes.read_byte().unwrap() {
                0 => Ok(Argument::Option(Box::new(None))),
                1 => {
                    let argument =
                        Argument::decode(chain, metadata, argument_meta.as_ref(), &mut *bytes)?;
                    Ok(Argument::Option(Box::new(Some(argument))))
                }
                _ => Err(DecodeError("Unexpected first byte for Option.".to_string())),
            },
            ArgumentMeta::Tuple(argument_metas) => {
                let mut result: Vec<Argument> = Vec::new();
                for argument_meta in argument_metas {
                    result.push(Argument::decode(
                        chain,
                        metadata,
                        argument_meta,
                        &mut *bytes,
                    )?);
                }
                Ok(Argument::Tuple(result))
            }
            ArgumentMeta::Primitive(name) => {
                if name == "sp_std::marker::PhantomData<(AccountId, Event)>" {
                    Err(ArgumentDecodeError::UnsupportedPrimitiveType(
                        name.to_string(),
                    ))
                } else if name == "<T::Lookup as StaticLookup>::Source" {
                    if metadata.is_signer_address_multi(chain) {
                        match MultiAddress::decode(&mut *bytes) {
                            Ok(multi_address) => {
                                Ok(Argument::Primitive(
                                    Box::new(ArgumentPrimitive::MultiAddress(multi_address))
                                ))
                            },
                            Err(_) => Err(ArgumentDecodeError::DecodeError(
                                "Cannot decode MultiAddress for <T::Lookup as StaticLookup>::Source."
                                    .to_string(),
                            )),
                        }
                    } else {
                        match AccountId::decode(&mut *bytes) {
                            Ok(account_id) => Ok(Argument::Primitive(Box::new(
                                ArgumentPrimitive::MultiAddress(MultiAddress::Id(account_id)),
                            ))),
                            Err(_) => Err(ArgumentDecodeError::DecodeError(
                                "Cannot decode AccountId for <T::Lookup as StaticLookup>::Source."
                                    .to_string(),
                            )),
                        }
                    }
                } else {
                    match ArgumentPrimitive::decode(name, &mut *bytes) {
                        Ok(argument_primitive) => {
                            Ok(Argument::Primitive(Box::new(argument_primitive)))
                        }
                        Err(error) => Err(error),
                    }
                }
            }
        }
    }
}
