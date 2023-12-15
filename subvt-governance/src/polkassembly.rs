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

pub async fn fetch_track_referenda(
    track_id: u16,
    page: u16,
    limit: u16,
) -> anyhow::Result<Vec<ReferendumPost>> {
    let http_client = get_http_client()?;
    let url = format!(
        "https://api.polkassembly.io/api/v1/listing/on-chain-posts?proposalType=referendums_v2&trackNo={track_id}&page={page}&listingLimit={limit}&sortBy=newest"
    );
    let response = http_client
        .get(url)
        .header("x-network", &CONFIG.substrate.chain)
        .send()
        .await?;
    let posts = response
        .json::<ReferendaQueryResponse>()
        .await?
        .posts
        .iter()
        .filter(|post| post.track_no == track_id)
        .cloned()
        .collect();
    Ok(posts)
}

pub async fn fetch_referendum_details(id: u32) -> anyhow::Result<ReferendumPostDetails> {
    let http_client = get_http_client()?;
    let response = http_client
        .get(format!(
            "https://api.polkassembly.io/api/v1/posts/on-chain-post?postId={}&proposalType=referendums_v2",
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
