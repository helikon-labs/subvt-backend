use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::db::PostgresPayoutStakersExtrinsic;
use subvt_types::app::extrinsic::PayoutStakersExtrinsic;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn get_payout_stakers_extrinsics_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<PayoutStakersExtrinsic>> {
        let db_extrinsics: Vec<PostgresPayoutStakersExtrinsic> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, is_nested_call, nesting_index, caller_account_id, validator_account_id, era_index, is_successful
            FROM sub_extrinsic_payout_stakers
            WHERE block_hash = $1 AND is_successful = true
            ORDER BY "id" ASC
            "#,
        )
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut extrinsics = Vec::new();
        for db_extrinsic in db_extrinsics {
            extrinsics.push(PayoutStakersExtrinsic::from(db_extrinsic)?)
        }
        Ok(extrinsics)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save_payout_stakers_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        maybe_nesting_index: &Option<String>,
        is_successful: bool,
        caller_account_id: &AccountId,
        validator_account_id: &AccountId,
        era_index: u32,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(caller_account_id).await?;
        self.save_account(validator_account_id).await?;
        let maybe_result: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_payout_stakers (block_hash, extrinsic_index, is_nested_call, nesting_index, caller_account_id, validator_account_id, era_index, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(block_hash, extrinsic_index, nesting_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(maybe_nesting_index)
            .bind(caller_account_id.to_string())
            .bind(validator_account_id.to_string())
            .bind(era_index as i64)
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
