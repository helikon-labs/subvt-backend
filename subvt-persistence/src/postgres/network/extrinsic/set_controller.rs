use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::db::PostgresSetControllerExtrinsic;
use subvt_types::app::extrinsic::SetControllerExtrinsic;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn get_set_controller_extrinsics_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<SetControllerExtrinsic>> {
        let db_extrinsics: Vec<PostgresSetControllerExtrinsic> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, is_nested_call, batch_index, caller_account_id, controller_account_id, is_successful
            FROM sub_extrinsic_set_controller
            WHERE block_hash = $1 AND is_successful = true
            ORDER BY "id" ASC
            "#,
        )
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut extrinsics = Vec::new();
        for db_extrinsic in db_extrinsics {
            extrinsics.push(SetControllerExtrinsic::from(db_extrinsic)?)
        }
        Ok(extrinsics)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save_set_controller_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        batch_index: &Option<String>,
        is_successful: bool,
        caller_account_id: &AccountId,
        controller_account_id: &AccountId,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(caller_account_id).await?;
        self.save_account(controller_account_id).await?;
        let maybe_result: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_set_controller (block_hash, extrinsic_index, is_nested_call, batch_index, caller_account_id, controller_account_id, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(block_hash, extrinsic_index, batch_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(batch_index)
            .bind(caller_account_id.to_string())
            .bind(controller_account_id.to_string())
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
