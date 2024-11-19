//! 1KV-related storage - for Polkadot and Kusama.
use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::dn::DNNode;

impl PostgreSQLNetworkStorage {
    pub async fn save_dn_node(
        &self,
        node: &DNNode,
        history_record_count: i64,
    ) -> anyhow::Result<i32> {
        let validator_account_id = AccountId::from_str(&node.stash)?;
        self.save_account(&validator_account_id).await?;
        let candidate_save_result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_onekv_candidate (validator_account_id, identity, status)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(validator_account_id.to_string())
        .bind(&node.identity)
        .bind(&node.status)
        .fetch_one(&self.connection_pool)
        .await?;

        // only keep the relevant number of candidate records
        let candidate_record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM sub_onekv_candidate
            WHERE validator_account_id = $1
            "#,
        )
        .bind(validator_account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        if candidate_record_count.0 > history_record_count {
            sqlx::query(
                r#"
                DELETE FROM sub_onekv_candidate
                WHERE id IN
                (
                    SELECT id FROM sub_onekv_candidate
                    WHERE validator_account_id = $1
                    ORDER BY id ASC
                    LIMIT $2
                )
                "#,
            )
            .bind(validator_account_id.to_string())
            .bind(candidate_record_count.0 - history_record_count)
            .execute(&self.connection_pool)
            .await?;
        }
        // return persisted record id
        Ok(candidate_save_result.0)
    }
}

impl PostgreSQLNetworkStorage {
    pub async fn get_dn_node_by_account_id(
        &self,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<DNNode>> {
        let maybe_dn_node: Option<(String, String, String)> = sqlx::query_as(
            r#"
            SELECT validator_account_id, identity, status
            FROM sub_onekv_candidate
            WHERE validator_account_id = $1
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
        .bind(account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(dn_node) = maybe_dn_node {
            Ok(Some(DNNode {
                stash: dn_node.0,
                identity: dn_node.1,
                status: dn_node.2,
            }))
        } else {
            Ok(None)
        }
    }
}

impl PostgreSQLNetworkStorage {
    pub async fn save_onekv_nominator(
        &self,
        stash: &str,
        history_record_count: i64,
    ) -> anyhow::Result<i32> {
        let account_id = AccountId::from_str(stash)?;
        self.save_account(&account_id).await?;
        let nominator_save_result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_onekv_nominator (account_id)
            VALUES ($1)
            RETURNING id
            "#,
        )
        .bind(account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;

        // only keep the relevant number of nominator records
        let nominator_record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM sub_onekv_nominator
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        if nominator_record_count.0 > history_record_count {
            sqlx::query(
                r#"
                DELETE FROM sub_onekv_nominator
                WHERE id IN
                (
                    SELECT id FROM sub_onekv_nominator
                    WHERE account_id = $1
                    ORDER BY id ASC
                    LIMIT $2
                )
                "#,
            )
            .bind(account_id.to_string())
            .bind(nominator_record_count.0 - history_record_count)
            .execute(&self.connection_pool)
            .await?;
        }
        // return persisted record id
        Ok(nominator_save_result.0)
    }

    pub async fn get_onekv_nominator_stash_account_ids(&self) -> anyhow::Result<Vec<AccountId>> {
        let db_account_ids: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT account_id
            FROM sub_onekv_nominator
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(db_account_ids
            .iter()
            .filter_map(|db_account_id| AccountId::from_str(&db_account_id.0).ok())
            .collect())
    }

    pub async fn is_onekv_nominator_account_id(
        &self,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM sub_onekv_nominator
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count.0 > 0)
    }

    pub async fn delete_onekv_candidate_records_older_than_days(
        &self,
        day_count: u8,
    ) -> anyhow::Result<()> {
        sqlx::query(
            format!(
                "DELETE FROM sub_onekv_candidate WHERE created_at < now() - interval '{day_count} days' RETURNING id",
            )
            .as_str(),
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}
