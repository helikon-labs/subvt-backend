use crate::CONFIG;
use subvt_types::governance::polkassembly::{
    GraphQLOperation, GraphQLQuery, GraphQLQueryVariables, ReferendaQueryResponse, ReferendumPost,
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
    let query = GraphQLQuery {
        operation: GraphQLOperation::ReferendumList,
        variables: GraphQLQueryVariables {
            id: None,
            limit: Some(10),
            post_type: Some(2),
        },
        query: r#"
            query AllReferendaPosts($postType: Int!, $limit: Int!) {
                posts(
                    limit: $limit
                    where: {
                        type: { id: { _eq: $postType } },
                        onchain_link: {
                            onchain_referendum_id: { _is_null: false },
                        }
                    }
                    order_by: { onchain_link: { onchain_referendum_id: desc } }
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
                            voteThreshold
                            referendumStatus(last: 1) {
                                id
                                status
                            }
                        }
                        proposer_address
                    }
                }
            }
        "#
        .to_string(),
    };
    let response = http_client
        .post(format!(
            "https://{}.polkassembly.io/v1/graphql",
            CONFIG.substrate.chain
        ))
        .body(serde_json::to_string(&query)?)
        .send()
        .await?;
    let posts = response.json::<ReferendaQueryResponse>().await?.data.posts;
    let mut result = vec![];
    for post in posts {
        if let Some(referendum) = post.onchain_link.onchain_referendum.first() {
            if let Some(status) = referendum.referendum_status.first() {
                if status.status.to_lowercase() == "started" {
                    result.push(post.clone());
                }
            }
        }
    }
    // increasing id
    result.reverse();
    Ok(result)
}

pub async fn fetch_referendum_details(id: u32) -> anyhow::Result<Option<ReferendumPost>> {
    let http_client = get_http_client()?;
    let query = GraphQLQuery {
        operation: GraphQLOperation::ReferendumDetails,
        variables: GraphQLQueryVariables {
            id: Some(id),
            limit: None,
            post_type: None,
        },
        query: r#"
            query ReferendumPostAndComments($id: Int!) {
                posts(
                    where: {
                        onchain_link: {
                            onchain_referendum_id: {_eq: $id}
                        }
                    }
                ) { ...referendumPost  }
            }
            fragment referendumPost on posts {
                author {
                    id
                    kusama_default_address
                    polkadot_default_address
                    username
                }
                content
                created_at
                id
                updated_at
                onchain_link {
                    ...onchainLinkReferendum
                }
                title
                type {
                    id
                    name
                }
            }
            fragment onchainLinkReferendum on onchain_links {
                id
                proposer_address
                onchain_referendum_id
                onchain_referendum(where: {}) {
                    id
                    delay
                    end
                    voteThreshold
                    referendumStatus(orderBy: id_DESC) {
                        blockNumber {
                            startDateTime
                            number
                        }
                        status
                        id
                    }
                    preimage {
                        hash
                        id
                        metaDescription
                        method
                        preimageArguments {
                            id
                            name
                            value
                        }
                    }
                }
            }
        "#
        .to_string(),
    };
    let response = http_client
        .post(format!(
            "https://{}.polkassembly.io/v1/graphql",
            CONFIG.substrate.chain
        ))
        .body(serde_json::to_string(&query)?)
        .send()
        .await?;
    let posts = response.json::<ReferendaQueryResponse>().await?.data.posts;
    if let Some(post) = posts.first() {
        Ok(Some(post.clone()))
    } else {
        Ok(None)
    }
}
