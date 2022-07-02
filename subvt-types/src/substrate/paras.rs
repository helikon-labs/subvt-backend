use parity_scale_codec::Decode;
use polkadot_runtime_parachains::scheduler::{AssignmentKind, CoreAssignment};
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
        write!(f, "{}", str)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ParaCoreAssignment {
    pub core_index: u32,
    pub para_id: u32,
    pub para_assignment_kind: ParaAssignmentKind,
    pub group_index: u32,
}

impl ParaCoreAssignment {
    pub fn from_core_assignment_vector_hex_string(hex_string: &str) -> anyhow::Result<Vec<Self>> {
        let mut bytes: &[u8] = &hex::decode(hex_string.trim_start_matches("0x"))?;
        let core_assignments: Vec<CoreAssignment> = Decode::decode(&mut bytes)?;
        let mut result = Vec::new();
        for core_assignment in core_assignments {
            let id: Id = serde_json::from_str(&serde_json::to_string(&core_assignment.para_id)?)?;
            let assignment = Self {
                core_index: core_assignment.core.0,
                para_id: id.0,
                para_assignment_kind: match core_assignment.kind {
                    AssignmentKind::Parachain => ParaAssignmentKind::Parachain,
                    AssignmentKind::Parathread(_, _) => ParaAssignmentKind::Parathread,
                },
                group_index: core_assignment.group_idx.0,
            };
            result.push(assignment)
        }
        Ok(result)
    }
}
