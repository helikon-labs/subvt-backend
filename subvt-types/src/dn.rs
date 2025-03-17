use frame_support::{Deserialize, Serialize};
use subvt_proc_macro::Diff;

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DNDataResponse {
    pub selected: Vec<DNNode>,
    pub nominators: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DNNode {
    pub stash: String,
    pub identity: String,
    pub status: String,
}
