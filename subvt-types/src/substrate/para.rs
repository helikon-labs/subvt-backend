use crate::substrate::legacy::LegacyCoreOccupied;
use crate::substrate::BlockNumber;
use polkadot_primitives::ScrapedOnChainVotes;
use polkadot_runtime_parachains::scheduler::CoreOccupied;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Id(pub u32);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ParaAssignmentKind {
    Parathread,
    Parachain,
}

impl Display for ParaAssignmentKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Parathread => "parathread",
            Self::Parachain => "parachain",
        };
        write!(f, "{str}")
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ParaCoreAssignment {
    pub core_index: u32,
    pub para_id: u32,
    pub group_index: u32,
}

impl ParaCoreAssignment {
    pub fn from_on_chain_votes_legacy(
        group_size: u8,
        cores: Vec<LegacyCoreOccupied>,
        votes: ScrapedOnChainVotes,
    ) -> anyhow::Result<Vec<Self>> {
        let mut result = Vec::new();
        for votes in votes.backing_validators_per_candidate {
            let para_id: u32 = votes.0.descriptor.para_id.into();
            if let Some(vote) = votes.1.first() {
                let group_index = vote.0 .0 / (group_size as u32);
                // get core index
                let mut maybe_core_index: Option<u32> = None;
                for (index, core) in cores.iter().enumerate() {
                    match core {
                        LegacyCoreOccupied::Paras(entry) => {
                            let core_para_id = entry.assignment.para_id.0;
                            if core_para_id == para_id {
                                maybe_core_index = Some(index as u32)
                            }
                        }
                        LegacyCoreOccupied::Free => (),
                    }
                }
                if let Some(core_index) = maybe_core_index {
                    let assignment = Self {
                        core_index,
                        para_id,
                        group_index,
                    };
                    result.push(assignment)
                }
            }
        }
        Ok(result)
    }

    pub fn from_on_chain_votes(
        group_size: u8,
        cores: Vec<CoreOccupied<BlockNumber>>,
        votes: ScrapedOnChainVotes,
    ) -> anyhow::Result<Vec<Self>> {
        let mut result = Vec::new();
        for votes in votes.backing_validators_per_candidate {
            let para_id: u32 = votes.0.descriptor.para_id.into();
            if let Some(vote) = votes.1.first() {
                let group_index = vote.0 .0 / (group_size as u32);
                // get core index
                let mut maybe_core_index: Option<u32> = None;
                for (index, core) in cores.iter().enumerate() {
                    match core {
                        CoreOccupied::Paras(entry) => {
                            let core_para_id: u32 = entry.assignment.para_id().into();
                            if core_para_id == para_id {
                                maybe_core_index = Some(index as u32)
                            }
                        }
                        CoreOccupied::Free => (),
                    }
                }
                if let Some(core_index) = maybe_core_index {
                    let assignment = Self {
                        core_index,
                        para_id,
                        group_index,
                    };
                    result.push(assignment)
                }
            }
        }
        Ok(result)
    }
}
