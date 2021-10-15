use crate::crypto::AccountId;
use pallet_election_provider_multi_phase::ElectionCompute;
use parity_scale_codec::{Compact, Decode};
use sp_npos_elections::ElectionScore;

pub type ValidatorIndex = u16;
pub type NominatorIndex = u32;
pub type ExtendedBalance = u128;
pub type SolutionSupports = Vec<(AccountId, SolutionSupport)>;

#[derive(Clone, Debug, Decode)]
pub struct ElectionSize {
    _validators: Compact<ValidatorIndex>,
    _nominators: Compact<NominatorIndex>,
}

#[derive(Clone, Debug, Decode)]
pub struct DefunctVoter {
    _who: AccountId,
    _vote_count: Compact<u32>,
    _candidate_count: Compact<u32>,
}

#[derive(Clone, Debug, Decode)]
pub struct ReadySolution {
    _supports: SolutionSupports,
    _score: ElectionScore,
    _compute: ElectionCompute,
}

#[derive(Clone, Debug, Decode)]
pub struct SolutionSupport {
    _total: ExtendedBalance,
    _voters: Vec<(AccountId, ExtendedBalance)>,
}
