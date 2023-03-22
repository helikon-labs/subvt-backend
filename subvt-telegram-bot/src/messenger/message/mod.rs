//! This module manages the creation of the content for every message type.
use crate::query::QueryType;
use subvt_types::crypto::AccountId;
use subvt_types::governance::polkassembly::{ReferendumPost, ReferendumPostDetails};
use subvt_types::onekv::OneKVCandidateSummary;
use subvt_types::sub_id::NFTCollection;
use subvt_types::substrate::democracy::ReferendumVote;
use subvt_types::subvt::{NetworkStatus, ValidatorDetails};
use subvt_types::telegram::{TelegramChatValidator, TelegramChatValidatorSummary};

pub mod content;

/// All types of messages that can be sent to a chat.
pub enum MessageType {
    About,
    Help,
    Intro,
    Ok,
    BadRequest,
    GenericError,
    Broadcast,
    BroadcastConfirm,
    UnknownCommand(String),
    InvalidAddress(String),
    InvalidAddressTryAgain(String),
    ValidatorNotFound {
        maybe_address: Option<String>,
    },
    AddValidatorNotFound(String),
    ValidatorExistsOnChat(String),
    TooManyValidatorsOnChat,
    NoValidatorsOnChat,
    ValidatorAdded,
    AddValidator,
    ValidatorList {
        validators: Vec<TelegramChatValidator>,
        query_type: QueryType,
    },
    ValidatorInfo {
        address: String,
        maybe_validator_details: Box<Option<ValidatorDetails>>,
        maybe_onekv_candidate_summary: Box<Option<OneKVCandidateSummary>>,
    },
    NominationSummary {
        chat_validator_id: u64,
        validator_details: ValidatorDetails,
    },
    NominationDetails {
        validator_details: ValidatorDetails,
        onekv_nominator_account_ids: Vec<AccountId>,
    },
    ValidatorRemoved(TelegramChatValidator),
    RemoveAllValidatorsConfirm,
    AllValidatorsRemoved,
    Settings,
    NetworkStatus(NetworkStatus),
    NoPayoutsFound,
    NoRewardsFound,
    NoOpenReferendaFound,
    ReferendumList(Vec<ReferendumPost>),
    ReferendumNotFound(u32),
    ReferendumDetails {
        post: ReferendumPostDetails,
        chat_validator_votes: Vec<(TelegramChatValidator, Option<ReferendumVote>)>,
    },
    SelectContactType,
    EnterBugReport,
    EnterFeatureRequest,
    ReportSaved,
    BugReport(String),
    FeatureRequest(String),
    NFTs {
        validator_id: u64,
        total_count: usize,
        collection_page: NFTCollection,
        page_index: usize,
        has_prev: bool,
        has_next: bool,
    },
    NoNFTsForValidator,
    Loading,
    ValidatorsSummary(Vec<TelegramChatValidatorSummary>),
}
