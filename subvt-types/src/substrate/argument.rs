//! Extrinsic and event arguments, and the decode logic for them.
use crate::{
    crypto::AccountId,
    substrate::{
        error::DecodeError,
        extrinsic::SubstrateExtrinsic,
        legacy::{
            DefunctVoter, ElectionSize, LegacyDispatchError, LegacyValidatorPrefs, ReadySolution,
            ValidatorIndex,
        },
        metadata::{ArgumentMeta, Metadata},
        CallHash, Chain, MultiAddress, OpaqueTimeSlot, ProxyType, RewardDestination, SlotRange,
        ValidatorPreferences,
    },
};
use frame_support::traits::schedule::v3::TaskName;
use frame_support::traits::schedule::LookupError;
use frame_support::weights::OldWeight;
use frame_support::{
    dispatch::{DispatchError, DispatchInfo, DispatchResult, Weight},
    traits::{
        schedule::{Period, Priority},
        BalanceStatus,
    },
};
use frame_system::{Key, KeyValue};
use pallet_bounties::BountyIndex;
use pallet_collective::{MemberCount, ProposalIndex};
use pallet_democracy::{AccountVote, Conviction, PropIndex, ReferendumIndex, VoteThreshold};
use pallet_election_provider_multi_phase::{ElectionCompute, SolutionOrSnapshotSize};
use pallet_elections_phragmen::Renouncing;
use pallet_gilt::ActiveIndex;
use pallet_identity::{Data, IdentityFields, IdentityInfo, Judgement, RegistrarIndex};
use pallet_im_online::sr25519::AuthorityId;
use pallet_im_online::Heartbeat;
use pallet_multisig::Timepoint;
use pallet_nomination_pools::{BondExtra, PoolId, PoolState};
use pallet_scheduler::TaskAddress;
use pallet_staking::{ConfigOp, Exposure, ValidatorPrefs};
use pallet_vesting::VestingInfo;
use parity_scale_codec::{Compact, Decode, Input};
use polkadot_core_primitives::{AccountIndex, CandidateHash, Hash, Header};
pub use polkadot_primitives::v2::BlockNumber;
use polkadot_primitives::v2::{
    Balance, CandidateReceipt, CoreIndex, GroupIndex, HeadData, HrmpChannelId, Id, InherentData,
    PvfCheckStatement, ValidationCode, ValidationCodeHash, ValidatorSignature,
};
use polkadot_runtime::MaxAdditionalFields;
use polkadot_runtime_common::assigned_slots::SlotLeasePeriodStart;
use polkadot_runtime_common::{
    auctions::AuctionIndex,
    claims::{EcdsaSignature, EthereumAddress, StatementKind},
};
use polkadot_runtime_parachains::disputes::slashing::DisputeProof;
use polkadot_runtime_parachains::disputes::{DisputeLocation, DisputeResult};
use polkadot_runtime_parachains::paras::ParaGenesisArgs;
use polkadot_runtime_parachains::ump::{MessageId, OverweightIndex};
use sp_consensus_babe::{digests::NextConfigDescriptor, EquivocationProof};
use sp_core::sr25519::Signature;
use sp_finality_grandpa::AuthorityList;
use sp_npos_elections::{ElectionScore, Supports, VoteWeight};
use sp_runtime::{MultiSignature, MultiSigner, Perbill, Percent, Perquintill};
use sp_session::MembershipProof;
use sp_staking::{offence::Kind, SessionIndex};

pub type IdentificationTuple = (AccountId, Exposure<AccountId, Balance>);

type EraIndex = u32;

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
    BagsListScore(VoteWeight),
    Balance(Balance),
    BalanceStatus(BalanceStatus),
    Bool(bool),
    BountyIndex(BountyIndex),
    BlockNumber(BlockNumber),
    Call(SubstrateExtrinsic),
    CallHash(CallHash),
    CandidateHash(CandidateHash),
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
    CompactMoment(Compact<u64>),
    CompactParachainId(Compact<Id>),
    CompactWeight(Compact<Weight>),
    ConfigOpAccountId(ConfigOp<AccountId>),
    ConfigOpBalance(ConfigOp<Balance>),
    ConfigOpPercent(ConfigOp<Percent>),
    ConfigOpPerbill(ConfigOp<Perbill>),
    ConfigOpU32(ConfigOp<u32>),
    CoreIndex(CoreIndex),
    DefunctVoter(DefunctVoter),
    DemocracyConviction(Conviction),
    DemocracyProposalIndex(PropIndex),
    DemocracyVoteThreshold(VoteThreshold),
    DemocracyAccountVote(AccountVote<Balance>),
    DispatchError(DispatchError),
    DispatchInfo(DispatchInfo),
    DispatchResult(DispatchResult),
    DisputeLocation(DisputeLocation),
    DisputeProof(Box<DisputeProof>),
    DisputeResult(DisputeResult),
    EcdsaSignature(EcdsaSignature),
    ElectionCompute(ElectionCompute),
    ElectionScore(ElectionScore),
    ElectionSize(ElectionSize),
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
    ImOnlineAuthorityId(AuthorityId),
    ImOnlineSignature(Signature),
    Key(Key),
    KeyOwnerProof(MembershipProof),
    KeyValue(KeyValue),
    MultiAsset(Box<xcm::v0::prelude::MultiAsset>),
    MultiAddress(MultiAddress),
    MultiLocationV0(xcm::v0::MultiLocation),
    MultiLocationV1(xcm::v1::MultiLocation),
    MultiLocationV2(xcm::v2::MultiLocation),
    MultisigTimepoint(Timepoint<BlockNumber>),
    MultiSignature(MultiSignature),
    MultiSigner(MultiSigner),
    NominationPoolBondExtra(BondExtra<Balance>),
    NominationPoolId(PoolId),
    NominationPoolState(PoolState),
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
    ParaGenesisArgs(ParaGenesisArgs),
    Perbill(Perbill),
    Percent(Percent),
    Perquintill(Perquintill),
    SlotLeasePeriodStart(SlotLeasePeriodStart),
    RewardDestination(RewardDestination),
    SchedulerLookupError(LookupError),
    U8(u8),
    U8Array32([u8; 32]),
    U16(u16),
    U32(u32),
    U64(u64),
    _PhantomData,
    ProxyType(ProxyType),
    PvfCheckStatement(PvfCheckStatement),
    ReadySolution(ReadySolution),
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
    ValidationCodeHash(ValidationCodeHash),
    ValidatorIndex(ValidatorIndex),
    ValidatorPreferences(ValidatorPreferences),
    ValidatorSignature(ValidatorSignature),
    VersionedMultiAssets(Box<xcm::VersionedMultiAssets>),
    VersionedMultiLocation(xcm::VersionedMultiLocation),
    VersionedXcm(Box<xcm::VersionedXcm<()>>),
    VestingInfo(VestingInfo<Balance, BlockNumber>),
    VoteWeight(VoteWeight),
    Weight(Weight),
    WeightLimit(xcm::v2::WeightLimit),
    XcmV0(xcm::v0::Xcm<()>),
    XcmV1(xcm::v1::Xcm<()>),
    XcmV2(xcm::v2::Xcm<()>),
    XcmError(xcm::latest::Error),
    XcmOutcome(Box<xcm::latest::Outcome>),
    XcmQueryId(xcm::prelude::QueryId),
    XcmResponse(xcm::prelude::Response),
    XcmV0Outcome(Box<xcm::v0::Outcome>),
    XcmVersion(xcm::prelude::XcmVersion),
    TaskName(TaskName),
    XcmWeight(xcm::latest::Weight),
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

pub fn extract_optional_argument_primitive(
    argument: &Argument,
) -> Result<Option<ArgumentPrimitive>, DecodeError> {
    match argument {
        Argument::Option(argument_option) => match &**argument_option {
            Some(argument) => Ok(Some(extract_argument_primitive(argument)?)),
            None => Ok(None),
        },
        _ => Err(DecodeError::Error(format!(
            "Cannot extract optional argument primitive: {:?}",
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

macro_rules! get_optional_argument_primitive {
    ($argument_expr: expr, $argument_primitive_type: ident) => {{
        let optional_argument_primitive =
            crate::substrate::argument::extract_optional_argument_primitive($argument_expr)?;
        match optional_argument_primitive {
            Some(ArgumentPrimitive::$argument_primitive_type(primitive)) => Ok(Some(primitive)),
            None => Ok(None),
            _ => Err(DecodeError::Error(format!(
                "Cannot get optional argument primitive {:?}.",
                optional_argument_primitive
            ))),
        }?
    }};
}
pub(crate) use get_optional_argument_primitive;

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
use crate::substrate::legacy::LegacyDispatchInfo;
pub(crate) use get_argument_vector;

macro_rules! generate_argument_primitive_decoder_impl {
    ([$(($name: literal, $decode_function_name: ident, $argument_primitive_enum_case_name: ident),)+]) => {
        impl ArgumentPrimitive {
            $(
                pub fn $decode_function_name<I: Input>(input: &mut I) -> Result<ArgumentPrimitive, ArgumentDecodeError> {
                    match Decode::decode(&mut *input) {
                        Ok(decoded) => Ok(ArgumentPrimitive::$argument_primitive_enum_case_name(decoded)),
                        Err(error) => Err(ArgumentDecodeError::DecodeError(format!("Cannot decode {}: {:?}", $name, error))),
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
    ("AccountId", decode_account_id_1, AccountId),
    ("T::AccountId", decode_account_id_2, AccountId),
    ("<T as frame_system::Config>::AccountId", decode_account_id_3, AccountId),
    ("AccountIndex", decode_account_index_1, AccountIndex),
    ("T::AccountIndex", decode_account_index_2, AccountIndex),
    ("AuctionIndex", decode_auction_index, AuctionIndex),
    ("NextConfigDescriptor", decode_babe_next_config_description, BABENextConfigDescriptor),
    ("AuthorityId", decode_authority_id_1, ImOnlineAuthorityId),
    ("T::AuthorityId", decode_authority_id_2, ImOnlineAuthorityId),
    ("Balance", decode_balance_1, Balance),
    ("BalanceOf<T>", decode_balance_2, Balance),
    ("BalanceOf<T, I>", decode_balance_3, Balance),
    ("T::Balance", decode_balance_4, Balance),
    ("Status", decode_balance_status, BalanceStatus),
    ("bool", decode_bool, Bool),
    ("BountyIndex", decode_bounty_index, BountyIndex),
    ("BlockNumber", decode_block_number, BlockNumber),
    ("T::BlockNumber", decode_t_block_number, BlockNumber),
    ("CallHash", decode_call_hash_1, CallHash),
    ("CallHashOf<T>", decode_call_hash_2, CallHash),
    ("CandidateHash", decode_candidate_hash, CandidateHash),
    ("CandidateReceipt<Hash>", decode_candidate_receipt_1, CandidateReceipt),
    ("CandidateReceipt<T::Hash>", decode_candidate_receipt_2, CandidateReceipt),
    ("MemberCount", decode_collective_member_count, CollectiveMemberCount),
    ("ProposalIndex", decode_collective_proposal_index, CollectiveProposalIndex),
    ("Compact<ActiveIndex>", decode_compact_gilt_active_index, CompactGiltActiveIndex),
    ("Compact<AuctionIndex>", decode_compact_auction_index, CompactAuctionIndex),
    ("Compact<BalanceOf<T>>", decode_compact_balance_1, CompactBalance),
    ("Compact<T::Balance>", decode_compact_balance_2, CompactBalance),
    ("Compact<BalanceOf<T, I>>", decode_compact_balance_4, CompactBalance),
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
    ("ConfigOp<T::AccountId>", decode_config_op_account_id, ConfigOpAccountId),
    ("ConfigOp<BalanceOf<T>>", decode_config_op_balance, ConfigOpBalance),
    ("ConfigOp<u32>", decode_config_op_u32, ConfigOpU32),
    ("ConfigOp<Percent>", decode_config_op_percent, ConfigOpPercent),
    ("ConfigOp<Perbill>", decode_config_op_perbill, ConfigOpPerbill),
    ("CoreIndex", decode_core_index, CoreIndex),
    ("Box<DisputeProof>", decode_dispute_proof, DisputeProof),
    ("DefunctVoter<<T::Lookup as StaticLookup>::Source>", decode_defunct_voter, DefunctVoter),
    ("Conviction", decode_democracy_conviction, DemocracyConviction),
    ("PropIndex", decode_democracy_proposal_index, DemocracyProposalIndex),
    ("VoteThreshold", decode_democracy_vote_threshold, DemocracyVoteThreshold),
    ("AccountVote<BalanceOf<T>>", decode_democracy_account_vote, DemocracyAccountVote),
    ("DisputeLocation", decode_dispute_location, DisputeLocation),
    ("DisputeResult", decode_dispute_result, DisputeResult),
    ("EcdsaSignature", decode_ecdsa_signature, EcdsaSignature),
    ("ElectionCompute", decode_election_compute, ElectionCompute),
    ("ElectionSize", decode_election_size, ElectionSize),
    ("ElectionScore", decode_election_score, ElectionScore),
    ("Supports<T::AccountId>", decode_election_supports, ElectionSupports),
    ("EquivocationProof<T::Hash, T::BlockNumber>", decode_equivocation_proof_1, EquivocationProof),
    ("Box<EquivocationProof<T::Hash, T::BlockNumber>>", decode_equivocation_proof_2, EquivocationProof),
    ("Box<EquivocationProof<T::Header>>", decode_equivocation_proof_3, EquivocationProof),
    ("EquivocationProof<T::Header>", decode_equivocation_proof_4, EquivocationProof),
    ("EraIndex", decode_era_index, EraIndex),
    ("EthereumAddress", decode_ethereum_address, EthereumAddress),
    ("ActiveIndex", decode_gilt_active_index, GiltActiveIndex),
    ("AuthorityList", decode_granpa_authority_list, GrandpaAuthorityList),
    ("GroupIndex", decode_group_index, GroupIndex),
    ("Hash", decode_hash_1, Hash),
    ("H256", decode_h256, U8Array32),
    ("T::Hash", decode_hash_2, Hash),
    ("T::Header", decode_header, Header),
    ("Heartbeat<T::BlockNumber>", decode_heartbeat, Heartbeat),
    ("IdentificationTuple", decode_identification_tuple_1, IdentificationTuple),
    ("IdentificationTuple<T>", decode_identification_tuple_2, IdentificationTuple),
    ("Data", decode_identity_data, IdentityData),
    ("IdentityFields", decode_identity_fields, IdentityFields),
    ("Box<IdentityInfo<T::MaxAdditionalFields>>", decode_identity_info_1, IdentityInfo),
    ("IdentityInfo", decode_identity_info_2, IdentityInfo),
    ("Judgement", decode_identity_judgement_1, IdentityJudgement),
    ("Judgement<BalanceOf<T>>", decode_identity_judgement_2, IdentityJudgement),
    ("<T::AuthorityId as RuntimeAppPublic>::Signature", decode_im_online_signature, ImOnlineSignature),
    ("Key", decode_key, Key),
    ("T::KeyOwnerProof", decode_key_owner_proof, KeyOwnerProof),
    ("KeyValue", decode_key_value, KeyValue),
    ("ParachainsInherentData<T::Header>", decode_parachains_inherent_data, ParachainsInherentData),
    ("MultiAsset", decode_multi_asset, MultiAsset),
    ("Timepoint<BlockNumber>", decode_multisig_timepoint_1, MultisigTimepoint),
    ("Timepoint<T::BlockNumber>", decode_multisig_timepoint_2, MultisigTimepoint),
    ("MultiSignature", decode_multisignature, MultiSignature),
    ("MultiSigner", decode_multisigner, MultiSigner),
    ("Kind", decode_offence_kind, OffenceKind),
    ("OpaqueTimeSlot", decode_opaque_time_slot, OpaqueTimeSlot),
    ("HeadData", decode_parachain_head_data, ParachainHeadData),
    ("HrmpChannelId", decode_parachain_hrmp_channel_id, ParachainHRMPChannelId),
    ("ParaId", decode_parachain_id, ParachainId),
    ("ParaGenesisArgs", decode_para_genesis_args, ParaGenesisArgs),
    ("LeasePeriod", decode_parachain_lease_period_1, ParachainLeasePeriod),
    ("LeasePeriodOf<T>", decode_parachain_lease_period_2, ParachainLeasePeriod),
    ("Compact<LeasePeriodOf<T>>", decode_parachain_compact_lease_period, ParachainCompactLeasePeriod),
    ("LookupError", decode_scheduler_lookup_error, SchedulerLookupError),
    ("MessageId", decode_parachain_ump_message_id, ParachainUMPMessageId),
    ("OverweightIndex", decode_parachain_ump_overweight_index, ParachainUMPOverweightIndex),
    ("Compact<T::Moment>", decode_compact_moment_1, CompactMoment),
    ("T::Moment", decode_compact_moment_2, CompactMoment),
    ("u8", decode_u8, U8),
    ("[u8; 32]", decode_u8_array_32, U8Array32),
    ("u16", decode_u16, U16),
    ("u32", decode_u32, U32),
    ("u64", decode_u64, U64),
    ("Perbill", decode_perbill, Perbill),
    ("Percent", decode_percent, Percent),
    ("Perquintill", decode_perquintill, Perquintill),
    ("BondExtra<BalanceOf<T>>", decode_nomination_pool_bond_extra, NominationPoolBondExtra),
    ("PoolId", decode_nomination_pool_id, NominationPoolId),
    ("PoolState", decode_nomination_pool_state, NominationPoolState),
    ("ProxyType", decode_proxy_type_1, ProxyType),
    ("T::ProxyType", decode_proxy_type_2, ProxyType),
    ("PvfCheckStatement", decode_pvf_check_statement, PvfCheckStatement),
    ("ReadySolution<T::AccountId>", decode_ready_solution, ReadySolution),
    ("ReferendumIndex", decode_referendum_index, ReferendumIndex),
    ("RegistrarIndex", decode_registrar_index, RegistrarIndex),
    ("Renouncing", decode_renouncing, Renouncing),
    ("RewardDestination<T::AccountId>", decode_reward_destination, RewardDestination),
    ("schedule::Period<T::BlockNumber>", decode_scheduler_period, SchedulerPeriod),
    ("schedule::Priority", decode_scheduler_priority, SchedulerPriority),
    ("TaskAddress<BlockNumber>", decode_scheduler_task_address_1, SchedulerTaskAddress),
    ("TaskAddress<T::BlockNumber>", decode_scheduler_task_address_2, SchedulerTaskAddress),
    ("SessionIndex", decode_session_index, SessionIndex),
    ("T::Score", decode_bags_list_score, BagsListScore),
    ("T::Keys", decode_session_keys, SessionKeys),
    ("SlotRange", decode_slot_range, SlotRange),
    ("SolutionOrSnapshotSize", decode_solution_or_snapshot_size, SolutionOrSnapshotSize),
    ("SlotLeasePeriodStart", decode_slot_lease_period_start, SlotLeasePeriodStart),
    ("StatementKind", decode_statement_kind, StatementKind),
    ("ValidationCode", decode_validation_code, ValidationCode),
    ("ValidationCodeHash", decode_validation_code_hash, ValidationCodeHash),
    ("ValidatorIndex", decode_validation_index, ValidatorIndex),
    ("ValidatorSignature", decode_validator_signature, ValidatorSignature),
    ("Box<VersionedMultiAssets>", decode_versioned_multi_assets_1, VersionedMultiAssets),
    ("VersionedMultiAssets", decode_versioned_multi_assets_2, VersionedMultiAssets),
    ("Box<VersionedMultiLocation>", decode_versioned_multi_location_1, VersionedMultiLocation),
    ("VersionedMultiLocation", decode_versioned_multi_location_2, VersionedMultiLocation),
    ("Box<VersionedXcm<()>>", decode_versioned_xcm_1, VersionedXcm),
    ("Box<VersionedXcm<T::Call>>", decode_versioned_xcm_2, VersionedXcm),
    ("Box<VersionedXcm<T::RuntimeCall>>", decode_versioned_xcm_3, VersionedXcm),
    ("Box<VersionedXcm<<T as SysConfig>::Call>>", decode_versioned_xcm_4, VersionedXcm),
    ("Box<VersionedXcm<<T as SysConfig>::RuntimeCall>>", decode_versioned_xcm_5, VersionedXcm),
    ("VestingInfo<BalanceOf<T>, T::BlockNumber>", decode_vesting_info, VestingInfo),
    ("VoteWeight", decode_vote_weight, VoteWeight),
    ("WeightLimit", decode_weight_limit, WeightLimit),
    ("XcmError", decode_xcm_error, XcmError),
    ("Outcome", decode_xcm_outcome_1, XcmOutcome),
    ("xcm::latest::Outcome", decode_xcm_outcome_2, XcmOutcome),
    ("xcm::v0::Outcome", decode_xcm_v0_outcome, XcmV0Outcome),
    ("XcmVersion", decode_xcm_version, XcmVersion),
    ("QueryId", decode_xcm_query_id, XcmQueryId),
    ("Response", decode_xcm_response, XcmResponse),
    ("TaskName", decode_task_name, TaskName),
    ("XcmWeight", decode_xcm_weight, XcmWeight),
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
    fn decode_validator_prefs(
        chain: &Chain,
        metadata: &Metadata,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Self, ArgumentDecodeError> {
        if metadata.is_validator_prefs_legacy(chain) {
            match LegacyValidatorPrefs::decode(&mut *bytes) {
                Ok(legacy_validator_prefs) => Ok(Argument::Primitive(Box::new(
                    ArgumentPrimitive::ValidatorPreferences(ValidatorPreferences {
                        commission_per_billion: legacy_validator_prefs.commission.deconstruct(),
                        blocks_nominations: false,
                    }),
                ))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode LegacyValidatorPrefs.".to_string(),
                )),
            }
        } else {
            match ValidatorPrefs::decode(&mut *bytes) {
                Ok(validator_prefs) => Ok(Argument::Primitive(Box::new(
                    ArgumentPrimitive::ValidatorPreferences(ValidatorPreferences {
                        commission_per_billion: validator_prefs.commission.deconstruct(),
                        blocks_nominations: validator_prefs.blocked,
                    }),
                ))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode ValidatorPrefs.".to_string(),
                )),
            }
        }
    }

    fn decode_multi_location(
        metadata: &Metadata,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Self, ArgumentDecodeError> {
        match metadata.get_xcm_version() {
            0 => match xcm::v0::MultiLocation::decode(&mut *bytes) {
                Ok(multi_location) => Ok(Argument::Primitive(Box::new(
                    ArgumentPrimitive::MultiLocationV0(multi_location),
                ))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode V0 MultiLocation.".to_string(),
                )),
            },
            1 => match xcm::v1::MultiLocation::decode(&mut *bytes) {
                Ok(multi_location) => Ok(Argument::Primitive(Box::new(
                    ArgumentPrimitive::MultiLocationV1(multi_location),
                ))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode V1 MultiLocation.".to_string(),
                )),
            },
            _ => match xcm::v2::MultiLocation::decode(&mut *bytes) {
                Ok(multi_location) => Ok(Argument::Primitive(Box::new(
                    ArgumentPrimitive::MultiLocationV2(multi_location),
                ))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode V2 MultiLocation.".to_string(),
                )),
            },
        }
    }

    fn decode_xcm(
        metadata: &Metadata,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Self, ArgumentDecodeError> {
        match metadata.get_xcm_version() {
            0 => match xcm::v0::Xcm::decode(&mut *bytes) {
                Ok(xcm) => Ok(Argument::Primitive(Box::new(ArgumentPrimitive::XcmV0(xcm)))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode V0 XCM.".to_string(),
                )),
            },
            1 => match xcm::v1::Xcm::decode(&mut *bytes) {
                Ok(xcm) => Ok(Argument::Primitive(Box::new(ArgumentPrimitive::XcmV1(xcm)))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode V1 XCM.".to_string(),
                )),
            },
            _ => match xcm::v2::Xcm::decode(&mut *bytes) {
                Ok(xcm) => Ok(Argument::Primitive(Box::new(ArgumentPrimitive::XcmV2(xcm)))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode V2 XCM.".to_string(),
                )),
            },
        }
    }

    fn decode_dispatch_error(
        metadata: &Metadata,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Self, ArgumentDecodeError> {
        if metadata.is_dispatch_error_legacy() {
            match LegacyDispatchError::decode(&mut *bytes) {
                Ok(legacy_dispatch_error) => Ok(Argument::Primitive(Box::new(
                    ArgumentPrimitive::DispatchError(legacy_dispatch_error.into()),
                ))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode legacy dispatch error.".to_string(),
                )),
            }
        } else {
            match DispatchError::decode(&mut *bytes) {
                Ok(dispatch_error) => Ok(Argument::Primitive(Box::new(
                    ArgumentPrimitive::DispatchError(dispatch_error),
                ))),
                Err(_) => Err(ArgumentDecodeError::DecodeError(
                    "Cannot decode dispatch error.".to_string(),
                )),
            }
        }
    }

    fn decode_dispatch_info(
        metadata: &Metadata,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Self, ArgumentDecodeError> {
        if metadata.is_weight_legacy() {
            let legacy_dispatch_info: LegacyDispatchInfo =
                Decode::decode(&mut *bytes).map_err(|error| {
                    ArgumentDecodeError::DecodeError(format!(
                        "Cannot decode legacy dispatch info: {:?}",
                        error
                    ))
                })?;
            let dispatch_info = DispatchInfo {
                weight: legacy_dispatch_info.weight.into(),
                class: legacy_dispatch_info.class,
                pays_fee: legacy_dispatch_info.pays_fee,
            };
            Ok(Argument::Primitive(Box::new(
                ArgumentPrimitive::DispatchInfo(dispatch_info),
            )))
        } else {
            Ok(Argument::Primitive(Box::new(
                ArgumentPrimitive::DispatchInfo(DispatchInfo::decode(&mut *bytes).map_err(
                    |error| {
                        ArgumentDecodeError::DecodeError(format!(
                            "Cannot decode dispatch info: {:?}",
                            error
                        ))
                    },
                )?),
            )))
        }
    }

    fn decode_dispatch_result(
        metadata: &Metadata,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Self, ArgumentDecodeError> {
        if metadata.is_dispatch_error_legacy() {
            let legacy_result: Result<(), LegacyDispatchError> = Decode::decode(&mut *bytes)
                .map_err(|error| {
                    ArgumentDecodeError::DecodeError(format!(
                        "Cannot decode legacy dispatch result: {:?}",
                        error
                    ))
                })?;
            let dispatch_result: DispatchResult = match legacy_result {
                Ok(()) => Ok(()),
                Err(legacy_error) => Err(legacy_error.into()),
            };
            Ok(Argument::Primitive(Box::new(
                ArgumentPrimitive::DispatchResult(dispatch_result),
            )))
        } else {
            Ok(Argument::Primitive(Box::new(
                ArgumentPrimitive::DispatchResult(DispatchResult::decode(&mut *bytes).map_err(
                    |error| {
                        ArgumentDecodeError::DecodeError(format!(
                            "Cannot decode dispatch result: {:?}",
                            error
                        ))
                    },
                )?),
            )))
        }
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn decode(
        chain: &Chain,
        metadata: &Metadata,
        argument_meta: &ArgumentMeta,
        extrinsic_signature: &Option<crate::substrate::extrinsic::Signature>,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Self, ArgumentDecodeError> {
        match argument_meta {
            ArgumentMeta::Vec(argument_meta) => {
                let length: Compact<u32> = match Decode::decode(bytes) {
                    Ok(length) => length,
                    Err(_) => {
                        return Err(ArgumentDecodeError::DecodeError(
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
                        extrinsic_signature,
                        &mut *bytes,
                    )?);
                }
                Ok(Argument::Vec(result))
            }
            ArgumentMeta::Option(argument_meta) => match bytes.read_byte().unwrap() {
                0 => Ok(Argument::Option(Box::new(None))),
                1 => {
                    let argument = Argument::decode(
                        chain,
                        metadata,
                        argument_meta.as_ref(),
                        extrinsic_signature,
                        &mut *bytes,
                    )?;
                    Ok(Argument::Option(Box::new(Some(argument))))
                }
                _ => Err(ArgumentDecodeError::DecodeError(
                    "Unexpected first byte for Option.".to_string(),
                )),
            },
            ArgumentMeta::Tuple(argument_metas) => {
                let mut result: Vec<Argument> = Vec::new();
                for argument_meta in argument_metas {
                    result.push(Argument::decode(
                        chain,
                        metadata,
                        argument_meta,
                        extrinsic_signature,
                        &mut *bytes,
                    )?);
                }
                Ok(Argument::Tuple(result))
            }
            ArgumentMeta::Primitive(name) => {
                if name == "sp_std::marker::PhantomData<(AccountId, Event)>"
                    || name == "Box<RawSolution<CompactOf<T>>>"
                    || name == "Box<RawSolution<SolutionOf<T>>>"
                    || name == "RawSolution<CompactOf<T>>"
                    || name == "CompactAssignments"
                    || name == "Box<T::PalletsOrigin>"
                    || name == "ChangesTrieConfiguration"
                    || name == "Box<CallOrHashOf<T>>"
                    || name == "Box<xcm::opaque::VersionedXcm>"
                    || name == "Box<RawSolution<SolutionOf<T::MinerConfig>>>"
                {
                    Err(ArgumentDecodeError::UnsupportedPrimitiveType(
                        name.to_string(),
                    ))
                } else if name == "Weight" {
                    if metadata.is_weight_legacy() {
                        let old_weight: OldWeight = match Decode::decode(bytes) {
                            Ok(old_weight) => old_weight,
                            Err(_) => {
                                return Err(ArgumentDecodeError::DecodeError(
                                    "Cannot decode OldWeight.".to_string(),
                                ))
                            }
                        };
                        Ok(Argument::Primitive(Box::new(ArgumentPrimitive::Weight(
                            Weight::from_ref_time(old_weight.0),
                        ))))
                    } else {
                        match Decode::decode(bytes) {
                            Ok(weight) => Ok(Argument::Primitive(Box::new(
                                ArgumentPrimitive::Weight(weight),
                            ))),
                            Err(_) => Err(ArgumentDecodeError::DecodeError(
                                "Cannot decode Weight.".to_string(),
                            )),
                        }
                    }
                } else if name == "Compact<Weight>" {
                    if metadata.is_weight_legacy() {
                        let compact_old_weight: Compact<OldWeight> = match Decode::decode(bytes) {
                            Ok(compact_old_weight) => compact_old_weight,
                            Err(_) => {
                                return Err(ArgumentDecodeError::DecodeError(
                                    "Cannot decode OldWeight.".to_string(),
                                ));
                            }
                        };
                        Ok(Argument::Primitive(Box::new(ArgumentPrimitive::Weight(
                            frame_support::dispatch::Weight::from(compact_old_weight.0),
                        ))))
                    } else {
                        match Decode::decode(bytes) {
                            Ok(weight) => Ok(Argument::Primitive(Box::new(
                                ArgumentPrimitive::Weight(weight),
                            ))),
                            Err(_) => Err(ArgumentDecodeError::DecodeError(
                                "Cannot decode Weight.".to_string(),
                            )),
                        }
                    }
                } else if name == "Box<<T as Config>::Call>"
                    || name == "Box<<T as Config>::RuntimeCall>"
                    || name == "Box<<T as Trait>::Call>"
                    || name == "Box<<T as Trait>::RuntimeCall>"
                    || name == "<T as Trait>::Call"
                    || name == "<T as Trait>::RuntimeCall"
                    || name == "<T as Config>::Call"
                    || name == "<T as Config>::RuntimeCall"
                    || name == "Box<<T as Config<I>>::Proposal>"
                    || name == "Box<<T as Trait<I>>::Proposal>"
                    || name == "OpaqueCall"
                    || name == "OpaqueCall<T>"
                {
                    if name == "OpaqueCall" || name == "OpaqueCall<T>" {
                        let vector_length_result: Result<Compact<u64>, _> =
                            Decode::decode(&mut *bytes);
                        match vector_length_result {
                            Ok(_) => (),
                            Err(_) => {
                                return Err(ArgumentDecodeError::DecodeError(format!(
                                    "Cannot decode byte vector length for {}.",
                                    name
                                )))
                            }
                        }
                    }
                    match SubstrateExtrinsic::decode_extrinsic(
                        chain,
                        metadata,
                        extrinsic_signature,
                        &mut *bytes,
                    ) {
                        Ok(extrinsic) => Ok(Argument::Primitive(Box::new(
                            ArgumentPrimitive::Call(extrinsic),
                        ))),
                        Err(decode_error) => Err(ArgumentDecodeError::DecodeError(format!(
                            "Cannot decode call type {}: {:?}",
                            name, decode_error
                        ))),
                    }
                } else if name == "<T::Lookup as StaticLookup>::Source"
                    || name == "AccountIdLookupOf<T>"
                {
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
                } else if name == "ValidatorPrefs" {
                    Argument::decode_validator_prefs(chain, metadata, &mut *bytes)
                } else if name == "MultiLocation" || name == "Box<MultiLocation>" {
                    Argument::decode_multi_location(metadata, &mut *bytes)
                } else if name == "Xcm<()>" || name == "Box<Xcm<T::Call>>" {
                    Argument::decode_xcm(metadata, &mut *bytes)
                } else if name == "DispatchInfo" {
                    Argument::decode_dispatch_info(metadata, &mut *bytes)
                } else if name == "DispatchError" {
                    Argument::decode_dispatch_error(metadata, &mut *bytes)
                } else if name == "DispatchResult" {
                    Argument::decode_dispatch_result(metadata, &mut *bytes)
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
