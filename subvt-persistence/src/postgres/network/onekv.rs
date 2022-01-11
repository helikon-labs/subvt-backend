//! 1KV-related storage - for Polkadot and Kusama.
use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;
use subvt_types::onekv::{OneKVCandidateDetails, OneKVValidity};

impl PostgreSQLNetworkStorage {
    pub async fn save_onekv_candidate(
        &self,
        candidate_details: &OneKVCandidateDetails,
        history_record_count: i64,
    ) -> anyhow::Result<i32> {
        let validator_account_id = AccountId::from_ss58_check(&candidate_details.stash_address)?;
        let kusama_account_id = if !candidate_details.kusama_stash_address.is_empty() {
            let kusama_account_id =
                AccountId::from_ss58_check(&candidate_details.kusama_stash_address)?;
            Some(kusama_account_id)
        } else {
            None
        };
        self.save_account(&validator_account_id).await?;
        let candidate_save_result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_onekv_candidate (onekv_id, validator_account_id, kusama_account_id, discovered_at, inclusion, last_valid, nominated_at, offline_accumulated, offline_since, online_since, name, location, rank, version, is_valid, score_updated_at, score_total, score_aggregate, score_inclusion, score_discovered, score_nominated, score_rank, score_unclaimed, score_bonded, score_faults, score_offline, score_randomness, score_span_inclusion, score_location, score_council_stake, score_democracy)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31)
            RETURNING id
            "#,
        )
            .bind(&candidate_details.id)
            .bind(validator_account_id.to_string())
            .bind(kusama_account_id.map(|account_id| account_id.to_string()))
            .bind(candidate_details.discovered_at as i64)
            .bind(candidate_details.inclusion)
            .bind(candidate_details.last_valid.map(|last_valid| last_valid as i64))
            .bind(candidate_details.nominated_at.map(|last_valid| last_valid as i64))
            .bind(candidate_details.offline_accumulated as i64)
            .bind(candidate_details.offline_since as i64)
            .bind(candidate_details.online_since as i64)
            .bind(&candidate_details.name)
            .bind(&candidate_details.location)
            .bind(candidate_details.rank)
            .bind(candidate_details.version.as_ref())
            .bind(candidate_details.is_valid())
            .bind(candidate_details.score.as_ref().map(|score| score.updated_at as i64))
            .bind(candidate_details.score.as_ref().map(|score| score.total))
            .bind(candidate_details.score.as_ref().map(|score| score.aggregate))
            .bind(candidate_details.score.as_ref().map(|score| score.inclusion))
            .bind(candidate_details.score.as_ref().map(|score| score.discovered))
            .bind(candidate_details.score.as_ref().map(|score| score.nominated))
            .bind(candidate_details.score.as_ref().map(|score| score.rank))
            .bind(candidate_details.score.as_ref().map(|score| score.unclaimed))
            .bind(candidate_details.score.as_ref().map(|score| score.bonded))
            .bind(candidate_details.score.as_ref().map(|score| score.faults))
            .bind(candidate_details.score.as_ref().map(|score| score.offline))
            .bind(candidate_details.score.as_ref().map(|score| score.randomness))
            .bind(candidate_details.score.as_ref().map(|score| score.span_inclusion))
            .bind(candidate_details.score.as_ref().map(|score| score.location))
            .bind(candidate_details.score.as_ref().map(|score| score.council_stake))
            .bind(candidate_details.score.as_ref().map(|score| score.democracy))
            .fetch_one(&self.connection_pool)
            .await?;

        // persist validity records and rank events
        let mut transaction = self.connection_pool.begin().await?;
        for validity in &candidate_details.validity {
            sqlx::query(
                r#"
                INSERT INTO sub_onekv_candidate_validity (onekv_id, onekv_candidate_id, validator_account_id, details, is_valid, ty, validity_updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO NOTHING
                "#,
            )
                .bind(&validity.id)
                .bind(candidate_save_result.0)
                .bind(validator_account_id.to_string())
                .bind(&validity.details)
                .bind(validity.is_valid)
                .bind(&validity.ty)
                .bind(validity.updated_at as i64)
                .execute(&mut transaction)
                .await?;
        }
        for rank_event in &candidate_details.rank_events {
            sqlx::query(
                r#"
                INSERT INTO sub_onekv_candidate_rank_event (onekv_id, validator_account_id, active_era, start_era, happened_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (onekv_id) DO NOTHING
                "#,
            )
                .bind(&rank_event.id)
                .bind(validator_account_id.to_string())
                .bind(rank_event.start_era as i32)
                .bind(rank_event.active_era as i32)
                .bind(rank_event.when as i64)
                .execute(&mut transaction)
                .await?;
        }
        for fault_event in &candidate_details.fault_events {
            sqlx::query(
                r#"
                INSERT INTO sub_onekv_candidate_fault_event (onekv_id, validator_account_id, previous_rank, reason, happened_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (onekv_id) DO NOTHING
                "#,
            )
                .bind(&fault_event.id)
                .bind(validator_account_id.to_string())
                .bind(fault_event.previous_rank.map(|previous_rank| previous_rank as i32))
                .bind(&fault_event.reason)
                .bind(fault_event.when as i64)
                .execute(&mut transaction)
                .await?;
        }
        transaction.commit().await?;

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

    pub async fn get_onekv_candidate_validity_items(
        &self,
        onekv_candidate_id: u32,
    ) -> anyhow::Result<Vec<OneKVValidity>> {
        let db_events: Vec<(String, String, bool, String, i64)> = sqlx::query_as(
            r#"
            SELECT onekv_id, details, is_valid, ty, validity_updated_at
            FROM sub_onekv_candidate_validity
            WHERE onekv_candidate_id = $1
            "#,
        )
        .bind(onekv_candidate_id as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(db_events
            .iter()
            .map(|db_validity| OneKVValidity {
                id: db_validity.0.clone(),
                details: db_validity.1.clone(),
                is_valid: db_validity.2,
                ty: db_validity.3.clone(),
                updated_at: db_validity.4 as u64,
            })
            .collect())
    }
}
