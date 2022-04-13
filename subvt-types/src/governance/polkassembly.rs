use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct AllReferendaQueryResponse {
    pub data: AllReferendaQueryResponseData,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AllReferendaQueryResponseData {
    pub posts: Vec<ReferendumPost>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendumPost {
    pub id: u32,
    pub author: Author,
    pub title: Option<String>,
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
pub struct OnchainReferendum {
    pub id: u32,
    pub end: u64,
    #[serde(rename = "referendumStatus")]
    pub referendum_status: Vec<OnchainReferendumStatus>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OnchainReferendumStatus {
    pub id: String,
    pub status: String,
}
