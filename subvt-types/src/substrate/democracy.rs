use crate::crypto::AccountId;
use crate::substrate::{Balance, DemocracyConviction};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Default)]
pub enum VoteType {
    #[default]
    Standard,
    Split,
    SplitAbstain,
}

impl Display for VoteType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            VoteType::Standard => "standard",
            VoteType::Split => "split",
            VoteType::SplitAbstain => "split_abstain",
        };
        write!(f, "{display}")
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DirectVote {
    pub ty: VoteType,
    pub aye: Option<Balance>,
    pub nay: Option<Balance>,
    pub abstain: Option<Balance>,
    pub conviction: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DelegatedVote {
    pub target_account_id: AccountId,
    pub balance: Balance,
    pub conviction: u8,
    pub delegate_account_id: AccountId,
    pub vote: DirectVote,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReferendumVote {
    pub account_id: AccountId,
    pub referendum_index: u32,
    pub direct_vote: Option<DirectVote>,
    pub delegated_vote: Option<DelegatedVote>,
}

pub fn get_democracy_conviction_u8(conviction: &DemocracyConviction) -> u8 {
    match conviction {
        DemocracyConviction::None => 0,
        DemocracyConviction::Locked1x => 1,
        DemocracyConviction::Locked2x => 2,
        DemocracyConviction::Locked3x => 3,
        DemocracyConviction::Locked4x => 4,
        DemocracyConviction::Locked5x => 5,
        DemocracyConviction::Locked6x => 6,
    }
}
