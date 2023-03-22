use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendaQueryResponse {
    pub count: u32,
    pub posts: Vec<ReferendumPost>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendumPost {
    pub post_id: u32,
    pub proposer: String,
    #[serde(rename = "title")]
    pub maybe_title: Option<String>,
    #[serde(rename = "method")]
    pub maybe_method: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendumPostDetails {
    pub post_id: u32,
    pub proposer: String,
    #[serde(rename = "title")]
    pub maybe_title: Option<String>,
    #[serde(rename = "method")]
    pub maybe_method: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "content")]
    pub maybe_content: Option<String>,
    #[serde(rename = "end")]
    pub end_block_number: u32,
    pub vote_threshold: String,
}
