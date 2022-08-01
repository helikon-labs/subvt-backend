#![warn(clippy::disallowed_types)]
use lazy_static::lazy_static;
use subvt_config::Config;
use subvt_types::sub_id::NFTCollection;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub async fn get_account_nfts(address: &str) -> anyhow::Result<NFTCollection> {
    let http_client: reqwest::Client = reqwest::Client::builder()
        .gzip(true)
        .brotli(true)
        .timeout(std::time::Duration::from_secs(
            CONFIG.http.request_timeout_seconds,
        ))
        .build()
        .unwrap();
    let collection: NFTCollection = http_client
        .get(&format!(
            "{}{}{}",
            CONFIG.sub_id.api_url, address, CONFIG.sub_id.nfts_path,
        ))
        .send()
        .await?
        .json()
        .await?;
    Ok(collection)
}
