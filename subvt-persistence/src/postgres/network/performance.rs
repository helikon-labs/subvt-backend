use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::performance::SessionValidatorPerformance;

impl PostgreSQLNetworkStorage {
    pub async fn get_session_validator_performance_updater_last_processed_session_id(
        &self,
    ) -> anyhow::Result<Option<u64>> {
        let row: (Option<i64>,) = sqlx::query_as(
            r#"
                SELECT MAX(session_index)
                FROM sub_session_validator_performance
                "#,
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(row.0.map(|index| index as u64))
    }

    pub async fn save_session_validator_performances(
        &self,
        performances: &[SessionValidatorPerformance],
    ) -> anyhow::Result<()> {
        let mut transaction = self.connection_pool.begin().await?;
        for (i, performance) in performances.iter().enumerate() {
            log::info!("Persist {} of {}.", i + 1, performances.len());
            sqlx::query(
                r#"
                INSERT INTO sub_session_validator_performance (validator_account_id, era_index, session_index, active_validator_index, authored_block_count, para_validator_group_index, para_validator_index, implicit_attestation_count, explicit_attestation_count, missed_attestation_count, attestations_per_billion)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT(validator_account_id, era_index, session_index) DO NOTHING
                RETURNING id
                "#,
            )
                .bind(performance.validator_account_id.to_string())
                .bind(performance.era_index as i64)
                .bind(performance.session_index as i64)
                .bind(performance.active_validator_index as i64)
                .bind(performance.authored_block_count as i32)
                .bind(performance.para_validator_group_index.map(|i| i as i64))
                .bind(performance.para_validator_index.map(|i| i as i64))
                .bind(performance.implicit_attestation_count.map(|i| i as i32))
                .bind(performance.explicit_attestation_count.map(|i| i as i32))
                .bind(performance.missed_attestation_count.map(|i| i as i32))
                .bind(performance.attestations_per_billion.map(|i| i as i32))
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
