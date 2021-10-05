use crate::{
    crypto::AccountId,
    substrate::metadata::ArgumentMeta,
    substrate::{CallHash, OpaqueTimeSlot},
};
use frame_support::{
    dispatch::{DispatchError, DispatchInfo, DispatchResult},
    traits::BalanceStatus,
    weights::Weight,
};
use log::debug;
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
use sp_authority_discovery::AuthorityId;
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
    AuthorityId(AuthorityId),
    Balance(Balance),
    BalanceStatus(BalanceStatus),
    Bool(bool),
    BountyIndex(BountyIndex),
    BlockNumber(BlockNumber),
    CallHash(CallHash),
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
    MultiLocation(xcm::latest::MultiLocation),
    MultisigTimepoint(Timepoint<BlockNumber>),
    OffenceKind(Kind),
    OpaqueTimeSlot(OpaqueTimeSlot),
    ParachainHeadData(HeadData),
    ParachainHRMPChannelId(HrmpChannelId),
    ParachainId(Id),
    ParachainLeasePeriod(BlockNumber),
    ParachainUMPMessageId(MessageId),
    U8(u8),
    U16(u16),
    U32(u32),
    _PhantomData,
    ProxyType(ProxyType),
    ReferendumIndex(ReferendumIndex),
    RegistrarIndex(RegistrarIndex),
    SchedulerTaskAddress(TaskAddress<BlockNumber>),
    SessionIndex(SessionIndex),
    Weight(Weight),
    Xcm(xcm::latest::Xcm<()>),
    XcmOutcome(Box<xcm::latest::Outcome>),
    XcmV0Outcome(Box<xcm::v0::Outcome>),
}

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
    ("AccountIndex", decode_account_index, AccountIndex),
    ("AuctionIndex", decode_auction_index, AuctionIndex),
    ("AuthorityId", decode_authority_id, AuthorityId),
    ("Balance", decode_balance, Balance),
    ("BalanceOf<T>", decode_balance_of_t, Balance),
    ("Status", decode_balance_status, BalanceStatus),
    ("bool", decode_bool, Bool),
    ("BountyIndex", decode_bounty_index, BountyIndex),
    ("BlockNumber", decode_block_number, BlockNumber),
    ("T::BlockNumber", decode_t_block_number, BlockNumber),
    ("CallHash", decode_call_hash, CallHash),
    ("DispatchInfo", decode_dispatch_info, DispatchInfo),
    ("DispatchError", decode_dispatch_error, DispatchError),
    ("DispatchResult", decode_dispatch_result, DispatchResult),
    ("CandidateReceipt<Hash>", decode_candidate_receipt, CandidateReceipt),
    ("CandidateReceipt<T::Hash>", decode_candidate_receipt_t, CandidateReceipt),
    ("MemberCount", decode_collective_member_count, CollectiveMemberCount),
    ("ProposalIndex", decode_collective_proposal_index, CollectiveProposalIndex),
    ("CoreIndex", decode_core_index, CoreIndex),
    ("PropIndex", decode_democracy_proposal_index, DemocracyProposalIndex),
    ("VoteThreshold", decode_democracy_vote_threshold, DemocracyVoteThreshold),
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
    ("u8", decode_u8, U8),
    ("u16", decode_u16, U16),
    ("u32", decode_u32, U32),
    ("ProxyType", decode_proxy_type, ProxyType),
    ("ReferendumIndex", decode_referendum_index, ReferendumIndex),
    ("RegistrarIndex", decode_registrar_index, RegistrarIndex),
    ("TaskAddress<BlockNumber>", decode_scheduler_task_address, SchedulerTaskAddress),
    ("SessionIndex", decode_session_index, SessionIndex),
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
                debug!("+-- decode vector(");
                for _ in 0..length.0 {
                    result.push(Argument::decode(argument_meta.as_ref(), &mut *bytes)?);
                }
                debug!("+-- ) end decode vector.");
                Ok(Argument::Vec(result))
            }
            ArgumentMeta::Option(argument_meta) => {
                debug!("+-- decode option:");
                match bytes.read_byte().unwrap() {
                    0 => Ok(Argument::Option(Box::new(None))),
                    1 => {
                        let argument = Argument::decode(argument_meta.as_ref(), &mut *bytes)?;
                        Ok(Argument::Option(Box::new(Some(argument))))
                    }
                    _ => Err(DecodeError("Unexpected first byte for Option.".to_string())),
                }
            }
            ArgumentMeta::Tuple(argument_metas) => {
                let mut result: Vec<Argument> = Vec::new();
                debug!("+-- decode tuple(");
                for argument_meta in argument_metas {
                    result.push(Argument::decode(argument_meta, &mut *bytes)?);
                }
                debug!("+-- ) end decode tuple.");
                Ok(Argument::Tuple(result))
            }
            ArgumentMeta::Primitive(name) => {
                if name == "sp_std::marker::PhantomData<(AccountId, Event)>" {
                    Err(ArgumentDecodeError::UnsupportedPrimitiveType(
                        name.to_string(),
                    ))
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
