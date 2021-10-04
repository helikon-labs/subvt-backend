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
                        ))
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
                let argument = if name == "AccountId" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::AccountId(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode AccountId.".to_string())),
                    }
                } else if name == "AccountIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::AccountIndex(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode AccountIndex.".to_string()))
                        }
                    }
                } else if name == "AuctionIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::AuctionIndex(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode AuctionIndex.".to_string()))
                        }
                    }
                } else if name == "AuthorityId" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::AuthorityId(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode AuthorityId.".to_string()))
                        }
                    }
                } else if name == "Balance" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::Balance(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode Balance.".to_string())),
                    }
                } else if name == "BalanceOf<T>" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::Balance(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode BalanceOf<T>.".to_string()))
                        }
                    }
                } else if name == "Status" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::BalanceStatus(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode Status.".to_string())),
                    }
                } else if name == "bool" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::Bool(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode bool.".to_string())),
                    }
                } else if name == "BountyIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::BountyIndex(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode BountyIndex.".to_string()))
                        }
                    }
                } else if name == "BlockNumber" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::BlockNumber(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode BlockNumber.".to_string()))
                        }
                    }
                } else if name == "T::BlockNumber" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::BlockNumber(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode T::BlockNumber.".to_string()))
                        }
                    }
                } else if name == "CallHash" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::CallHash(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode CallHash.".to_string())),
                    }
                } else if name == "DispatchInfo" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::DispatchInfo(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode DispatchInfo.".to_string()))
                        }
                    }
                } else if name == "DispatchError" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::DispatchError(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode DispatchError.".to_string()))
                        }
                    }
                } else if name == "DispatchResult" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::DispatchResult(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode DispatchResult.".to_string()))
                        }
                    }
                } else if name == "CandidateReceipt<Hash>" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::CandidateReceipt(decoded),
                        Err(_) => {
                            return Err(DecodeError(
                                "Cannot decode CandidateReceipt<Hash>.".to_string(),
                            ))
                        }
                    }
                } else if name == "CandidateReceipt<T::Hash>" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::CandidateReceipt(decoded),
                        Err(_) => {
                            return Err(DecodeError(
                                "Cannot decode CandidateReceipt<T::Hash>.".to_string(),
                            ))
                        }
                    }
                } else if name == "MemberCount" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::CollectiveMemberCount(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode MemberCount.".to_string()))
                        }
                    }
                } else if name == "ProposalIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::CollectiveProposalIndex(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode ProposalIndex.".to_string()))
                        }
                    }
                } else if name == "CoreIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::CoreIndex(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode CoreIndex.".to_string())),
                    }
                } else if name == "PropIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::DemocracyProposalIndex(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode PropIndex.".to_string())),
                    }
                } else if name == "VoteThreshold" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::DemocracyVoteThreshold(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode VoteThreshold.".to_string()))
                        }
                    }
                } else if name == "ElectionCompute" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::ElectionCompute(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode ElectionCompute.".to_string()))
                        }
                    }
                } else if name == "EraIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::EraIndex(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode EraIndex.".to_string())),
                    }
                } else if name == "EthereumAddress" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::EthereumAddress(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode EthereumAddress.".to_string()))
                        }
                    }
                } else if name == "ActiveIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::GiltActiveIndex(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode ActiveIndex.".to_string()))
                        }
                    }
                } else if name == "AuthorityList" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::GrandpaAuthorityList(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode AuthorityList.".to_string()))
                        }
                    }
                } else if name == "GroupIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::GroupIndex(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode GroupIndex.".to_string())),
                    }
                } else if name == "Hash" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::Hash(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode Hash.".to_string())),
                    }
                } else if name == "IdentificationTuple" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::IdentificationTuple(decoded),
                        Err(_) => {
                            return Err(DecodeError(
                                "Cannot decode IdentificationTuple.".to_string(),
                            ))
                        }
                    }
                } else if name == "MultiLocation" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::MultiLocation(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode MultiLocation.".to_string()))
                        }
                    }
                } else if name == "Timepoint<BlockNumber>" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::MultisigTimepoint(decoded),
                        Err(_) => {
                            return Err(DecodeError(
                                "Cannot decode Timepoint<BlockNumber>.".to_string(),
                            ))
                        }
                    }
                } else if name == "Kind" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::OffenceKind(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode Kind.".to_string())),
                    }
                } else if name == "OpaqueTimeSlot" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::OpaqueTimeSlot(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode OpaqueTimeSlot.".to_string()))
                        }
                    }
                } else if name == "HeadData" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::ParachainHeadData(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode HeadData.".to_string())),
                    }
                } else if name == "HrmpChannelId" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::ParachainHRMPChannelId(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode HrmpChannelId.".to_string()))
                        }
                    }
                } else if name == "ParaId" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::ParachainId(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode ParaId.".to_string())),
                    }
                } else if name == "LeasePeriod" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::ParachainLeasePeriod(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode LeasePeriod.".to_string()))
                        }
                    }
                } else if name == "MessageId" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::ParachainUMPMessageId(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode MessageId.".to_string())),
                    }
                } else if name == "u8" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::U8(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode u8.".to_string())),
                    }
                } else if name == "u16" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::U16(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode u16.".to_string())),
                    }
                } else if name == "u32" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::U32(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode u32.".to_string())),
                    }
                } else if name == "sp_std::marker::PhantomData<(AccountId, Event)>" {
                    return Err(UnsupportedPrimitiveType(name.clone()));
                } else if name == "ProxyType" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::ProxyType(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode ProxyType.".to_string())),
                    }
                } else if name == "ReferendumIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::ReferendumIndex(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode ReferendumIndex.".to_string()))
                        }
                    }
                } else if name == "RegistrarIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::RegistrarIndex(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode RegistrarIndex.".to_string()))
                        }
                    }
                } else if name == "TaskAddress<BlockNumber>" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::SchedulerTaskAddress(decoded),
                        Err(_) => {
                            return Err(DecodeError(
                                "Cannot decode TaskAddress<BlockNumber>.".to_string(),
                            ))
                        }
                    }
                } else if name == "SessionIndex" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::SessionIndex(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode SessionIndex.".to_string()))
                        }
                    }
                } else if name == "Weight" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::Weight(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode Weight.".to_string())),
                    }
                } else if name == "Xcm<()>" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::Xcm(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode Xcm<()>.".to_string())),
                    }
                } else if name == "Outcome" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::XcmOutcome(decoded),
                        Err(_) => return Err(DecodeError("Cannot decode Outcome.".to_string())),
                    }
                } else if name == "xcm::v0::Outcome" {
                    match Decode::decode(&mut *bytes) {
                        Ok(decoded) => ArgumentPrimitive::XcmV0Outcome(decoded),
                        Err(_) => {
                            return Err(DecodeError("Cannot decode xcm::v0::Outcome.".to_string()))
                        }
                    }
                } else {
                    return Err(UnknownPrimitiveType(name.clone()));
                };
                debug!("+-- decoded {}.", name);
                Ok(Argument::Primitive(Box::new(argument)))
            }
        }
    }
}
