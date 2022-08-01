use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::sub_id::{NFTChain, NFTCollection, NFT};

impl PostgreSQLNetworkStorage {
    pub async fn save_nft_collection(
        &self,
        owner_account_id: &AccountId,
        collection: &NFTCollection,
    ) -> anyhow::Result<()> {
        self.save_account(owner_account_id).await?;
        for (chain, nfts) in collection.iter() {
            for nft in nfts {
                sqlx::query(
                    r#"
                    INSERT INTO sub_nft(id, chain, owner_account_id, content_type, name, description, url, image_url)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT(id, chain, owner_account_id) DO UPDATE
                    SET content_type = EXCLUDED.content_type, name = EXCLUDED.name, description = EXCLUDED.description, url = EXCLUDED.url, image_url = EXCLUDED.image_url, updated_at = now()
                    "#,
                )
                .bind(&nft.id)
                .bind(chain.to_string())
                .bind(owner_account_id.to_string())
                .bind(&nft.content_type)
                .bind(&nft.name)
                .bind(&nft.description)
                .bind(&nft.url)
                .bind(&nft.image_url)
                .execute(&self.connection_pool)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn get_account_nft_count(
        &self,
        owner_account_id: &AccountId,
    ) -> anyhow::Result<usize> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT (id, chain, owner_account_id)) FROM sub_nft
            WHERE owner_account_id = $1
            "#,
        )
        .bind(owner_account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(result.0 as usize)
    }

    #[allow(clippy::type_complexity)]
    pub async fn get_account_nfts(
        &self,
        owner_account_id: &AccountId,
        page_index: usize,
        page_size: usize,
    ) -> anyhow::Result<NFTCollection> {
        let mut collection = NFTCollection::default();
        let records: Vec<(String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, chain, owner_account_id, content_type, name, description, url, image_url FROM sub_nft
            WHERE owner_account_id = $1
            ORDER BY chain ASC, name ASC, id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(owner_account_id.to_string())
        .bind(page_size as i64)
        .bind((page_size * page_index) as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        for record in records {
            let chain = NFTChain::from_str(&record.1)?;
            let nft = NFT {
                id: record.0,
                content_type: record.3,
                name: record.4,
                description: record.5,
                url: record.6,
                image_url: record.7,
            };
            if let Some(asd) = collection.get_mut(&chain) {
                asd.push(nft);
            } else {
                collection.insert(chain, vec![nft]);
            }
        }
        Ok(collection)
    }
}
