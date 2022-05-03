use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    #[allow(clippy::too_many_arguments)]
    pub async fn save_nominate_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        batch_index: &Option<String>,
        is_successful: bool,
        controller_account_id: &AccountId,
        validator_account_ids: &[AccountId],
    ) -> anyhow::Result<()> {
        self.save_account(controller_account_id).await?;
        let maybe_extrinsic_nominate_id: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_nominate (block_hash, extrinsic_index, is_nested_call, batch_index, controller_account_id, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT(block_hash, extrinsic_index, batch_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(batch_index)
            .bind(controller_account_id.to_string())
            .bind(is_successful)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(extrinsic_nominate_id) = maybe_extrinsic_nominate_id {
            for validator_account_id in validator_account_ids {
                self.save_account(validator_account_id).await?;
                sqlx::query(
                    r#"
                    INSERT INTO sub_extrinsic_nominate_validator (extrinsic_nominate_id, validator_account_id)
                    VALUES ($1, $2)
                    ON CONFLICT (extrinsic_nominate_id, validator_account_id) DO NOTHING
                    "#)
                    .bind(extrinsic_nominate_id.0)
                    .bind(validator_account_id.to_string())
                    .execute(&self.connection_pool)
                    .await?;
            }
        }
        Ok(())
    }
}
