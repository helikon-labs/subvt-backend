use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub enum GraphQLOperation {
    #[serde(rename(serialize = "AllReferendaPosts"))]
    ReferendumList,
    #[serde(rename(serialize = "ReferendumPostAndComments"))]
    ReferendumDetails,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLQueryVariables {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_type: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLQuery {
    #[serde(rename(serialize = "operationName"))]
    pub operation: GraphQLOperation,
    pub variables: GraphQLQueryVariables,
    pub query: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendaQueryResponse {
    pub data: ReferendaQueryResponseData,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendaQueryResponseData {
    pub posts: Vec<ReferendumPost>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendumPost {
    pub id: u32,
    pub author: Author,
    #[serde(rename = "title")]
    pub maybe_title: Option<String>,
    #[serde(rename = "content")]
    pub maybe_content: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub onchain_link: OnchainReferendumLink,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Author {
    pub id: u32,
    pub kusama_default_address: String,
    pub polkadot_default_address: String,
    pub username: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OnchainReferendumLink {
    pub id: u32,
    pub onchain_referendum_id: u32,
    pub onchain_referendum: Vec<OnchainReferendum>,
    pub proposer_address: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnchainReferendum {
    pub id: u32,
    #[serde(rename = "end")]
    pub end_block_number: u64,
    pub referendum_status: Vec<OnchainReferendumStatus>,
    pub vote_threshold: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OnchainReferendumStatus {
    pub id: String,
    pub status: String,
}
