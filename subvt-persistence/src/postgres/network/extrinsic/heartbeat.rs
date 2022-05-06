use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    #[allow(clippy::too_many_arguments)]
    pub async fn save_heartbeat_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        maybe_nesting_index: &Option<String>,
        is_successful: bool,
        block_number: u32,
        session_index: u32,
        validator_index: u32,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_heartbeat (block_hash, extrinsic_index, is_nested_call, nesting_index, block_number, session_index, validator_index, validator_account_id, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (block_hash, extrinsic_index, nesting_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(maybe_nesting_index)
            .bind(block_number as i64)
            .bind(session_index as i64)
            .bind(validator_index as i64)
            .bind(validator_account_id.to_string())
            .bind(is_successful)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }
}
