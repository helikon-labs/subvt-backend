use crate::substrate::error::DecodeError;
use crate::substrate::extrinsic::{Signature, SubstrateExtrinsic};
use crate::substrate::Balance;
use pallet_conviction_voting::AccountVote;
use pallet_democracy::ReferendumIndex;
use parity_scale_codec::{Compact, Decode};

const REMOVE_VOTE: &str = "remove_vote";
const VOTE: &str = "vote";

#[derive(Clone, Debug)]
pub enum ConvictionVotingExtrinsic {
    RemoveVote {
        maybe_signature: Option<Signature>,
        class: Option<u16>,
        index: ReferendumIndex,
    },
    Vote {
        maybe_signature: Option<Signature>,
        poll_index: Compact<ReferendumIndex>,
        vote: AccountVote<Balance>,
    },
}

impl ConvictionVotingExtrinsic {
    pub fn decode(
        name: &str,
        maybe_signature: &Option<Signature>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateExtrinsic>, DecodeError> {
        let maybe_event = match name {
            REMOVE_VOTE => Some(SubstrateExtrinsic::ConvictionVoting(
                ConvictionVotingExtrinsic::RemoveVote {
                    maybe_signature: maybe_signature.clone(),
                    class: Decode::decode(bytes)?,
                    index: Decode::decode(bytes)?,
                },
            )),
            VOTE => Some(SubstrateExtrinsic::ConvictionVoting(
                ConvictionVotingExtrinsic::Vote {
                    maybe_signature: maybe_signature.clone(),
                    poll_index: Decode::decode(bytes)?,
                    vote: Decode::decode(bytes)?,
                },
            )),
            _ => None,
        };
        Ok(maybe_event)
    }
}
