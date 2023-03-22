use crate::CONFIG;
use subvt_types::governance::polkassembly::{
    ReferendaQueryResponse, ReferendumPost, ReferendumPostDetails,
};

fn get_http_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .gzip(true)
        .brotli(true)
        .timeout(std::time::Duration::from_secs(
            CONFIG.http.request_timeout_seconds,
        ))
        .build()?)
}

pub async fn fetch_open_referendum_list() -> anyhow::Result<Vec<ReferendumPost>> {
    let http_client = get_http_client()?;
    let response = http_client
        .get("https://api.polkassembly.io/api/v1/listing/on-chain-posts?page=1&proposalType=referendums&listingLimit=10&sortBy=newest")
        .header(
            "x-network",
            &CONFIG.substrate.chain,
        )
        .send()
        .await?;
    let posts = response.json::<ReferendaQueryResponse>().await?.posts;
    let mut result = vec![];
    for post in posts {
        if post.status.to_lowercase() == "started" {
            result.push(post.clone());
        }
    }
    Ok(result)
}

pub async fn fetch_referendum_details(id: u32) -> anyhow::Result<ReferendumPostDetails> {
    let http_client = get_http_client()?;
    let response = http_client
        .get(format!(
            "https://api.polkassembly.io/api/v1/posts/on-chain-post?postId={}&proposalType=referendums",
            id,
        ))
        .header(
            "x-network",
            &CONFIG.substrate.chain,
        )
        .send()
        .await?;
    Ok(response.json::<ReferendumPostDetails>().await?)
}
