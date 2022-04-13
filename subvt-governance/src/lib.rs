use lazy_static::lazy_static;
use subvt_config::Config;
use subvt_types::governance::polkassembly::{AllReferendaQueryResponse, ReferendumPost};

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub async fn fetch_open_referendum_list() -> anyhow::Result<Vec<ReferendumPost>> {
    let http_client: reqwest::Client = reqwest::Client::builder()
        .gzip(true)
        .brotli(true)
        .timeout(std::time::Duration::from_secs(
            CONFIG.onekv.request_timeout_seconds,
        ))
        .build()
        .unwrap();

    let response = http_client
        .post("https://kusama.polkassembly.io/v1/graphql")
        .body(
            r#"{
	"operationName" : "AllReferendaPosts",
	"variables" : {"limit":25,"postType":2},
	"query" : "query AllReferendaPosts($postType: Int!, $limit: Int! = 5) {
        posts(
            limit: $limit
            where: {type: {id: {_eq: $postType}},
            onchain_link: {onchain_referendum_id: {_is_null: false}}}
            order_by: {onchain_link: {onchain_referendum_id: desc}}
        ) {
            id
            author {
                id
                kusama_default_address
                polkadot_default_address
                username
            }
            title
            created_at
            updated_at
            onchain_link {
                id
                onchain_referendum_id
                onchain_referendum(where: {}) {
                    id
                    end
                    referendumStatus(last: 1) {
                        id
                        status
                    }
                }
                proposer_address
            }
        }
    }"
}
        "#,
        )
        .send()
        .await?;
    Ok(response
        .json::<AllReferendaQueryResponse>()
        .await?
        .data
        .posts)
}
