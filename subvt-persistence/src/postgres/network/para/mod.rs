use crate::postgres::network::PostgreSQLNetworkStorage;
use sqlx::{Postgres, QueryBuilder};
use subvt_types::{
    crypto::AccountId, report::SessionParaValidator, substrate::para::ParaCoreAssignment,
};

impl PostgreSQLNetworkStorage {
    pub async fn get_session_para_validator(
        &self,
        session_index: u64,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<Option<SessionParaValidator>> {
        let session_para_validator: Option<(i64, i64)> = sqlx::query_as(
            r#"
            SELECT para_validator_group_index, para_validator_index
            FROM sub_session_para_validator
            WHERE session_index = $1
            AND validator_account_id = $2
            "#,
        )
        .bind(session_index as i64)
        .bind(validator_account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(session_para_validator.map(|ev| SessionParaValidator {
            session_index,
            validator_account_id: *validator_account_id,
            para_validator_group_index: ev.0 as u64,
            para_validator_index: ev.1 as u64,
        }))
    }

    pub async fn save_session_para_validators(
        &self,
        era_index: u32,
        session_index: u64,
        session_para_validators: &[(AccountId, u32, u32, u32)],
    ) -> anyhow::Result<()> {
        for chunk in session_para_validators.chunks(250) {
            {
                let mut query_builder = QueryBuilder::new("INSERT INTO sub_account (id)");
                query_builder.push_values(chunk, |mut query, validator| {
                    query.push_bind(validator.0.to_string());
                });
                query_builder.push(" ON CONFLICT (id) DO NOTHING");
                let query: sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments> =
                    query_builder.build();
                query.execute(&self.connection_pool).await?;
            }
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO sub_session_para_validator (era_index, session_index, validator_account_id, active_validator_index, para_validator_group_index, para_validator_index) ",
            );
            query_builder.push_values(chunk, |mut query, session_para_validator| {
                query
                    .push_bind(era_index as i64)
                    .push_bind(session_index as i64)
                    .push_bind(session_para_validator.0.to_string())
                    .push_bind(session_para_validator.1 as i64)
                    .push_bind(session_para_validator.2 as i64)
                    .push_bind(session_para_validator.3 as i64);
            });
            query_builder.push(
                r#"
                ON CONFLICT(session_index, validator_account_id) DO UPDATE SET
                    active_validator_index = EXCLUDED.active_validator_index,
                    para_validator_group_index = EXCLUDED.para_validator_group_index,
                    para_validator_index = EXCLUDED.para_validator_index
            "#,
            );
            let query: sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments> =
                query_builder.build();
            query.execute(&self.connection_pool).await?;
        }
        Ok(())
    }

    pub async fn save_para_core_assignment(
        &self,
        block_hash: &str,
        assignment: &ParaCoreAssignment,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
                INSERT INTO sub_para_core_assignment (block_hash, para_core_index, para_id, para_assignment_kind, para_validator_group_index)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT(block_hash, para_core_index) DO UPDATE
                SET para_id = EXCLUDED.para_id, para_assignment_kind = EXCLUDED.para_assignment_kind, para_validator_group_index = EXCLUDED.para_validator_group_index
                RETURNING id
                "#,
        )
            .bind(block_hash)
            .bind(assignment.core_index as i64)
            .bind(assignment.para_id as i64)
            .bind("")
            .bind(assignment.group_index as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0)
    }

    pub async fn get_min_para_vote_session_index(&self) -> anyhow::Result<u64> {
        let min_session_index: (i64,) = sqlx::query_as(
            r#"
            SELECT MIN(session_index)
            FROM sub_para_vote
            "#,
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(min_session_index.0 as u64)
    }

    pub async fn save_para_vote(
        &self,
        block_hash: &str,
        session_index: u32,
        para_id: u32,
        para_validator_index: u32,
        is_explicit: Option<bool>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
                INSERT INTO sub_para_vote (block_hash, session_index, para_id, para_validator_index, is_explicit)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT(block_hash, para_id, para_validator_index) DO UPDATE
                SET is_explicit = EXCLUDED.is_explicit
                RETURNING id
                "#,
        )
            .bind(block_hash)
            .bind(session_index as i64)
            .bind(para_id as i64)
            .bind(para_validator_index as i64)
            .bind(is_explicit)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0)
    }
}
