use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::db::PostgresValidateExtrinsic;
use subvt_types::app::extrinsic::ValidateExtrinsic;
use subvt_types::{crypto::AccountId, substrate::ValidatorPreferences};

impl PostgreSQLNetworkStorage {
    pub async fn get_validate_extrinsics_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<ValidateExtrinsic>> {
        let db_extrinsics: Vec<PostgresValidateExtrinsic> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, is_nested_call, nesting_index, stash_account_id, controller_account_id, commission_per_billion, blocks_nominations, is_successful
            FROM sub_extrinsic_validate
            WHERE block_hash = $1 AND is_successful = true
            ORDER BY "id" ASC
            "#,
        )
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut extrinsics = Vec::new();
        for db_extrinsic in db_extrinsics {
            extrinsics.push(ValidateExtrinsic::from(db_extrinsic)?)
        }
        Ok(extrinsics)
    }

    pub async fn save_validate_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        maybe_nesting_index: &Option<String>,
        is_successful: bool,
        (stash_account_id, controller_account_id): (&AccountId, &AccountId),
        validator_preferences: &ValidatorPreferences,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(stash_account_id).await?;
        self.save_account(controller_account_id).await?;
        let maybe_result: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_validate (block_hash, extrinsic_index, is_nested_call, nesting_index, stash_account_id, controller_account_id, commission_per_billion, blocks_nominations, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(maybe_nesting_index)
            .bind(stash_account_id.to_string())
            .bind(controller_account_id.to_string())
            .bind(validator_preferences.commission_per_billion as i64)
            .bind(validator_preferences.blocks_nominations)
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
