use crate::{
    crypto::AccountId,
    substrate::{
        error::DecodeError,
        extrinsic::SubstrateExtrinsic,
        metadata::{ArgumentMeta, Metadata},
        CallHash, Chain, MultiAddress, OpaqueTimeSlot, RewardDestination, SlotRange,
    },
};
use frame_support::{
    dispatch::{DispatchError, DispatchInfo, DispatchResult},
    traits::{
        schedule::{Period, Priority},
        BalanceStatus,
    },
    weights::Weight,
};
use frame_system::{Key, KeyValue};
use pallet_bounties::BountyIndex;
use pallet_collective::{MemberCount, ProposalIndex};
use pallet_democracy::{AccountVote, Conviction, PropIndex, ReferendumIndex, VoteThreshold};
use pallet_election_provider_multi_phase::{ElectionCompute, SolutionOrSnapshotSize};
use pallet_elections_phragmen::Renouncing;
use pallet_gilt::ActiveIndex;
use pallet_identity::{Data, IdentityFields, IdentityInfo, Judgement, RegistrarIndex};
use pallet_im_online::Heartbeat;
use pallet_multisig::{OpaqueCall, Timepoint};
use pallet_scheduler::TaskAddress;
use pallet_staking::{EraIndex, Exposure, ValidatorPrefs};
use pallet_vesting::VestingInfo;
use parity_scale_codec::{Compact, Decode, Input};
use polkadot_core_primitives::{AccountIndex, Hash, Header};
use polkadot_primitives::v1::{
    Balance, BlockNumber, CandidateReceipt, CoreIndex, GroupIndex, HeadData, HrmpChannelId, Id,
    InherentData, ValidationCode,
};
use polkadot_runtime::{MaxAdditionalFields, ProxyType};
use polkadot_runtime_common::{
    auctions::AuctionIndex,
    claims::{EcdsaSignature, EthereumAddress, StatementKind},
};
use polkadot_runtime_parachains::ump::{MessageId, OverweightIndex};
use sp_consensus_babe::{digests::NextConfigDescriptor, EquivocationProof};
use sp_core::{sr25519::Signature, ChangesTrieConfiguration};
use sp_finality_grandpa::AuthorityList;
use sp_npos_elections::{ElectionScore, Supports};
use sp_runtime::{MultiSignature, MultiSigner, Perbill, Percent, Perquintill};
use sp_session::MembershipProof;
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
    BABENextConfigDescriptor(NextConfigDescriptor),
    Balance(Balance),
    BalanceStatus(BalanceStatus),
    Bool(bool),
    BountyIndex(BountyIndex),
    BlockNumber(BlockNumber),
    Call(SubstrateExtrinsic),
    CallHash(CallHash),
    ChangesTrieConfiguration(ChangesTrieConfiguration),
    CompactAuctionIndex(Compact<AuctionIndex>),
    CompactBalance(Compact<Balance>),
    CompactBlockNumber(Compact<BlockNumber>),
    CompactBountyIndex(Compact<BountyIndex>),
    CompactEraIndex(Compact<EraIndex>),
    CompactCollectiveProposalIndex(Compact<ProposalIndex>),
    CompactDemocracyProposalIndex(Compact<PropIndex>),
    CompactGiltActiveIndex(Compact<ActiveIndex>),
    CompactMemberCount(Compact<MemberCount>),
    CompactPerquintill(Compact<Perquintill>),
    CompactRegistrarIndex(Compact<RegistrarIndex>),
    CompactReferendumIndex(Compact<ReferendumIndex>),
    CompactU32(Compact<u32>),
    CandidateReceipt(CandidateReceipt),
    CollectiveMemberCount(MemberCount),
    CollectiveProposalIndex(ProposalIndex),
    CompactParachainId(Compact<Id>),
    CompactWeight(Compact<Weight>),
    CoreIndex(CoreIndex),
    DemocracyConviction(Conviction),
    DemocracyProposalIndex(PropIndex),
    DemocracyVoteThreshold(VoteThreshold),
    DemocracyAccountVote(AccountVote<Balance>),
    DispatchError(DispatchError),
    DispatchInfo(DispatchInfo),
    DispatchResult(DispatchResult),
    EcdsaSignature(EcdsaSignature),
    ElectionCompute(ElectionCompute),
    ElectionScore(ElectionScore),
    ElectionSupports(Supports<AccountId>),
    EquivocationProof(Box<EquivocationProof<Header>>),
    EraIndex(EraIndex),
    EthereumAddress(EthereumAddress),
    GiltActiveIndex(ActiveIndex),
    GrandpaAuthorityList(AuthorityList),
    GroupIndex(GroupIndex),
    Hash(Hash),
    Header(Header),
    Heartbeat(Heartbeat<BlockNumber>),
    IdentificationTuple(IdentificationTuple),
    IdentityData(Data),
    IdentityFields(IdentityFields),
    IdentityInfo(Box<IdentityInfo<MaxAdditionalFields>>),
    IdentityJudgement(Judgement<Balance>),
    ImOnlineSignature(Signature),
    Key(Key),
    KeyOwnerProof(MembershipProof),
    KeyValue(KeyValue),
    Moment(Compact<u64>),
    MultiAsset(xcm::v0::prelude::MultiAsset),
    MultiAddress(MultiAddress),
    MultiLocation(xcm::latest::MultiLocation),
    MultisigOpaqueCall(OpaqueCall),
    MultisigTimepoint(Timepoint<BlockNumber>),
    MultiSignature(MultiSignature),
    MultiSigner(MultiSigner),
    OffenceKind(Kind),
    OpaqueTimeSlot(OpaqueTimeSlot),
    ParachainsInherentData(InherentData),
    ParachainHeadData(HeadData),
    ParachainHRMPChannelId(HrmpChannelId),
    ParachainId(Id),
    ParachainLeasePeriod(BlockNumber),
    ParachainCompactLeasePeriod(Compact<BlockNumber>),
    ParachainUMPMessageId(MessageId),
    ParachainUMPOverweightIndex(OverweightIndex),
    Perbill(Perbill),
    Percent(Percent),
    RewardDestination(RewardDestination),
    U8(u8),
    U8Array32([u8; 32]),
    U16(u16),
    U32(u32),
    U64(u64),
    _PhantomData,
    ProxyType(ProxyType),
    ReferendumIndex(ReferendumIndex),
    RegistrarIndex(RegistrarIndex),
    Renouncing(Renouncing),
    SchedulerPeriod(Period<BlockNumber>),
    SchedulerPriority(Priority),
    SchedulerTaskAddress(TaskAddress<BlockNumber>),
    SessionIndex(SessionIndex),
    SessionKeys([u8; 192]),
    SlotRange(SlotRange),
    SolutionOrSnapshotSize(SolutionOrSnapshotSize),
    StatementKind(StatementKind),
    ValidationCode(ValidationCode),
    ValidatorPrefs(ValidatorPrefs),
    VersionedMultiAssets(Box<xcm::VersionedMultiAssets>),
    VersionedMultiLocation(xcm::VersionedMultiLocation),
    VersionedXcm(Box<xcm::VersionedXcm<()>>),
    VestingInfo(VestingInfo<Balance, BlockNumber>),
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
                pub fn $decode_function_name<I: Input>(input: &mut I) -> Result<ArgumentPrimitive, ArgumentDecodeError> {
                    match Decode::decode(&mut *input) {
                        Ok(decoded) => Ok(ArgumentPrimitive::$argument_primitive_enum_case_name(decoded)),
                        Err(_) => Err(ArgumentDecodeError::DecodeError(format!("Cannot decode {}.", $name))),
                    }
                }
            )+
            pub fn decode<I: Input>(name: &str, bytes: &mut I) -> Result<ArgumentPrimitive, ArgumentDecodeError> {
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
    ("AccountIndex", decode_account_index_1, AccountIndex),
    ("T::AccountIndex", decode_account_index_2, AccountIndex),
    ("AuctionIndex", decode_auction_index, AuctionIndex),
    ("NextConfigDescriptor", decode_babe_next_config_description, BABENextConfigDescriptor),
    ("AuthorityId", decode_authority_id, AccountId),
    ("Balance", decode_balance_1, Balance),
    ("BalanceOf<T>", decode_balance_2, Balance),
    ("BalanceOf<T, I>", decode_balance_3, Balance),
    ("Status", decode_balance_status, BalanceStatus),
    ("bool", decode_bool, Bool),
    ("BountyIndex", decode_bounty_index, BountyIndex),
    ("BlockNumber", decode_block_number, BlockNumber),
    ("T::BlockNumber", decode_t_block_number, BlockNumber),
    ("CallHash", decode_call_hash_1, CallHash),
    ("CallHashOf<T>", decode_call_hash_2, CallHash),
    ("ChangesTrieConfiguration", decode_changes_trie_configuration, ChangesTrieConfiguration),
    ("CandidateReceipt<Hash>", decode_candidate_receipt_1, CandidateReceipt),
    ("CandidateReceipt<T::Hash>", decode_candidate_receipt_2, CandidateReceipt),
    ("MemberCount", decode_collective_member_count, CollectiveMemberCount),
    ("ProposalIndex", decode_collective_proposal_index, CollectiveProposalIndex),
    ("Compact<ActiveIndex>", decode_compact_gilt_active_index, CompactGiltActiveIndex),
    ("Compact<AuctionIndex>", decode_compact_auction_index, CompactAuctionIndex),
    ("Compact<BalanceOf<T>>", decode_compact_balance_1, CompactBalance),
    ("Compact<T::Balance>", decode_compact_balance_2, CompactBalance),
    ("Compact<BalanceOf<T, I>>", decode_compact_balance_3, CompactBalance),
    ("Compact<T::BlockNumber>", decode_compact_block_number, CompactBlockNumber),
    ("Compact<BountyIndex>", decode_compact_bounty_index, CompactBountyIndex),
    ("Compact<EraIndex>", decode_compact_era_index, CompactEraIndex),
    ("Compact<ParaId>", decode_compact_parachain_id, CompactParachainId),
    ("Compact<ProposalIndex>", decode_compact_collective_proposal_index, CompactCollectiveProposalIndex),
    ("Compact<PropIndex>", decode_compact_democracy_proposal_index, CompactDemocracyProposalIndex),
    ("Compact<MemberCount>", decode_compact_member_count, CompactMemberCount),
    ("Compact<Perquintill>", decode_compact_perquintill, CompactPerquintill),
    ("Compact<RegistrarIndex>", decode_compact_registrar_index, CompactRegistrarIndex),
    ("Compact<ReferendumIndex>", decode_compact_referendum_index, CompactReferendumIndex),
    ("Compact<u32>", decode_compact_u32, CompactU32),
    ("Compact<Weight>", decode_compact_weight, CompactWeight),
    ("CoreIndex", decode_core_index, CoreIndex),
    ("Conviction", decode_democracy_conviction, DemocracyConviction),
    ("PropIndex", decode_democracy_proposal_index, DemocracyProposalIndex),
    ("VoteThreshold", decode_democracy_vote_threshold, DemocracyVoteThreshold),
    ("AccountVote<BalanceOf<T>>", decode_democracy_account_vote, DemocracyAccountVote),
    ("DispatchInfo", decode_dispatch_info, DispatchInfo),
    ("DispatchError", decode_dispatch_error, DispatchError),
    ("DispatchResult", decode_dispatch_result, DispatchResult),
    ("EcdsaSignature", decode_ecdsa_signature, EcdsaSignature),
    ("ElectionCompute", decode_election_compute, ElectionCompute),
    ("ElectionScore", decode_election_score, ElectionScore),
    ("Supports<T::AccountId>", decode_election_supports, ElectionSupports),
    ("Box<EquivocationProof<T::Hash, T::BlockNumber>>", decode_equivocation_proof_1, EquivocationProof),
    ("Box<EquivocationProof<T::Header>>", decode_equivocation_proof_2, EquivocationProof),
    ("EraIndex", decode_era_index, EraIndex),
    ("EthereumAddress", decode_ethereum_address, EthereumAddress),
    ("ActiveIndex", decode_gilt_active_index, GiltActiveIndex),
    ("AuthorityList", decode_granpa_authority_list, GrandpaAuthorityList),
    ("GroupIndex", decode_group_index, GroupIndex),
    ("Hash", decode_hash_1, Hash),
    ("T::Hash", decode_hash_2, Hash),
    ("T::Header", decode_header, Header),
    ("Heartbeat<T::BlockNumber>", decode_heartbeat, Heartbeat),
    ("IdentificationTuple", decode_identification_tuple, IdentificationTuple),
    ("Data", decode_identity_data, IdentityData),
    ("IdentityFields", decode_identity_fields, IdentityFields),
    ("Box<IdentityInfo<T::MaxAdditionalFields>>", decode_identity_info, IdentityInfo),
    ("Judgement", decode_identity_judgement_1, IdentityJudgement),
    ("Judgement<BalanceOf<T>>", decode_identity_judgement_2, IdentityJudgement),
    ("<T::AuthorityId as RuntimeAppPublic>::Signature", decode_im_online_signature, ImOnlineSignature),
    ("Key", decode_key, Key),
    ("T::KeyOwnerProof", decode_key_owner_proof, KeyOwnerProof),
    ("KeyValue", decode_key_value, KeyValue),
    ("ParachainsInherentData<T::Header>", decode_parachains_inherent_data, ParachainsInherentData),
    ("MultiAsset", decode_multi_asset, MultiAsset),
    ("MultiLocation", decode_multi_location, MultiLocation),
    ("OpaqueCall", decode_multisig_opaque_call, MultisigOpaqueCall),
    ("Timepoint<BlockNumber>", decode_multisig_timepoint_1, MultisigTimepoint),
    ("Timepoint<T::BlockNumber>", decode_multisig_timepoint_2, MultisigTimepoint),
    ("MultiSignature", decode_multisignature, MultiSignature),
    ("MultiSigner", decode_multisigner, MultiSigner),
    ("Kind", decode_offence_kind, OffenceKind),
    ("OpaqueTimeSlot", decode_opaque_time_slot, OpaqueTimeSlot),
    ("HeadData", decode_parachain_head_data, ParachainHeadData),
    ("HrmpChannelId", decode_parachain_hrmp_channel_id, ParachainHRMPChannelId),
    ("ParaId", decode_parachain_id, ParachainId),
    ("LeasePeriod", decode_parachain_lease_period_1, ParachainLeasePeriod),
    ("LeasePeriodOf<T>", decode_parachain_lease_period_2, ParachainLeasePeriod),
    ("Compact<LeasePeriodOf<T>>", decode_parachain_compact_lease_period, ParachainCompactLeasePeriod),
    ("MessageId", decode_parachain_ump_message_id, ParachainUMPMessageId),
    ("OverweightIndex", decode_parachain_ump_overweight_index, ParachainUMPOverweightIndex),
    ("Compact<T::Moment>", decode_moment, Moment),
    ("u8", decode_u8, U8),
    ("[u8; 32]", decode_u8_array_32, U8Array32),
    ("u16", decode_u16, U16),
    ("u32", decode_u32, U32),
    ("u64", decode_u64, U64),
    ("Perbill", decode_perbill, Perbill),
    ("Percent", decode_percent, Percent),
    ("ProxyType", decode_proxy_type_1, ProxyType),
    ("T::ProxyType", decode_proxy_type_2, ProxyType),
    ("ReferendumIndex", decode_referendum_index, ReferendumIndex),
    ("RegistrarIndex", decode_registrar_index, RegistrarIndex),
    ("Renouncing", decode_renouncing, Renouncing),
    ("RewardDestination<T::AccountId>", decode_reward_destination, RewardDestination),
    ("schedule::Period<T::BlockNumber>", decode_scheduler_period, SchedulerPeriod),
    ("schedule::Priority", decode_scheduler_priority, SchedulerPriority),
    ("TaskAddress<BlockNumber>", decode_scheduler_task_address, SchedulerTaskAddress),
    ("SessionIndex", decode_session_index, SessionIndex),
    ("T::Keys", decode_session_keys, SessionKeys),
    ("SlotRange", decode_slot_range, SlotRange),
    ("SolutionOrSnapshotSize", decode_solution_or_snapshot_size, SolutionOrSnapshotSize),
    ("StatementKind", decode_statement_kind, StatementKind),
    ("ValidationCode", decode_validation_code, ValidationCode),
    ("ValidatorPrefs", decode_validator_prefs, ValidatorPrefs),
    ("Box<VersionedMultiAssets>", decode_versioned_multi_assets, VersionedMultiAssets),
    ("Box<VersionedMultiLocation>", decode_versioned_multi_location, VersionedMultiLocation),
    ("Box<VersionedXcm<()>>", decode_versioned_xcm, VersionedXcm),
    ("VestingInfo<BalanceOf<T>, T::BlockNumber>", decode_vesting_info, VestingInfo),
    ("Weight", decode_weight, Weight),
    ("Xcm<()>", decode_xcm, Xcm),
    ("Outcome", decode_xcm_outcome_1, XcmOutcome),
    ("xcm::latest::Outcome", decode_xcm_outcome_2, XcmOutcome),
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
                if name == "sp_std::marker::PhantomData<(AccountId, Event)>"
                    || name == "Box<RawSolution<CompactOf<T>>>"
                    || name == "Box<RawSolution<SolutionOf<T>>>"
                {
                    Err(ArgumentDecodeError::UnsupportedPrimitiveType(
                        name.to_string(),
                    ))
                } else if name == "Box<<T as Config>::Call>"
                    || name == "<T as Config>::Call"
                    || name == "Box<Xcm<T::Call>>"
                    || name == "Box<VersionedXcm<T::Call>>"
                    || name == "Box<<T as Config<I>>::Proposal>"
                {
                    match SubstrateExtrinsic::decode_extrinsic(chain, metadata, false, &mut *bytes)
                    {
                        Ok(extrinsic) => Ok(Argument::Primitive(Box::new(
                            ArgumentPrimitive::Call(extrinsic),
                        ))),
                        Err(decode_error) => Err(ArgumentDecodeError::DecodeError(format!(
                            "Cannot decode call type {}: {:?}",
                            name, decode_error
                        ))),
                    }
                } else if name == "<T::Lookup as StaticLookup>::Source" {
                    if metadata.is_signer_address_multi(chain) {
                        match MultiAddress::decode(&mut *bytes) {
                            Ok(multi_address) => {
                                Ok(Argument::Primitive(
                                    Box::new(ArgumentPrimitive::MultiAddress(multi_address))
                                ))
                            }
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
