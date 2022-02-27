//! 1KV-related storage - for Polkadot and Kusama.
use crate::postgres::network::PostgreSQLNetworkStorage;
use chrono::NaiveDateTime;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::onekv::{
    OneKVCandidateDetails, OneKVCandidateSummary, OneKVNominator, OneKVValidity,
};

type PostgresCandidateSummary = (
    i32,
    String,
    i64,
    String,
    Option<i64>,
    i64,
    i64,
    Option<i64>,
    Option<f64>,
    Option<f64>,
    Option<i64>,
    Option<String>,
    Option<String>,
    i64,
    Vec<String>,
    NaiveDateTime,
);

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
            INSERT INTO sub_onekv_candidate (onekv_id, validator_account_id, kusama_account_id, discovered_at, inclusion, span_inclusion, bonded, commission, is_active, reward_destination, telemetry_id, node_refs, unclaimed_eras, last_valid, nominated_at, offline_accumulated, offline_since, online_since, name, location, rank, version, is_valid, democracy_vote_count, democracy_votes, council_stake, council_votes, score_updated_at, score_total, score_aggregate, score_inclusion, score_discovered, score_nominated, score_rank, score_unclaimed, score_bonded, score_faults, score_offline, score_randomness, score_span_inclusion, score_location, score_council_stake, score_democracy)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42, $43)
            RETURNING id
            "#,
        )
            .bind(&candidate_details.id)
            .bind(validator_account_id.to_string())
            .bind(kusama_account_id.map(|account_id| account_id.to_string()))
            .bind(candidate_details.discovered_at as i64)
            .bind(candidate_details.inclusion)
            .bind(candidate_details.span_inclusion)
            .bind(candidate_details.bonded.map(|bonded| bonded.to_string()))
            .bind(candidate_details.commission)
            .bind(candidate_details.is_active)
            .bind(&candidate_details.reward_destination)
            .bind(candidate_details.telemetry_id)
            .bind(candidate_details.node_refs)
            .bind(&candidate_details.unclaimed_eras)
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
            .bind(candidate_details.democracy_vote_count)
            .bind(&candidate_details.democracy_votes)
            .bind(&candidate_details.council_stake)
            .bind(&candidate_details.council_votes)
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

impl PostgreSQLNetworkStorage {
    pub async fn get_onekv_candidate_summary_by_id(
        &self,
        id: u32,
    ) -> anyhow::Result<Option<OneKVCandidateSummary>> {
        let maybe_candidate_summary: Option<PostgresCandidateSummary> = sqlx::query_as(
            r#"
            SELECT id, onekv_id, discovered_at, name, nominated_at, online_since, offline_since, rank, score_total, score_aggregate, telemetry_id, version, location, democracy_vote_count, council_votes, created_at
            FROM sub_onekv_candidate
            WHERE id = $1
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
        .bind(id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(summary) = maybe_candidate_summary {
            Ok(Some(OneKVCandidateSummary {
                record_id: summary.0 as u32,
                onekv_id: summary.1,
                discovered_at: summary.2 as u64,
                name: summary.3,
                nominated_at: summary.4.map(|t| t as u64),
                online_since: summary.5 as u64,
                offline_since: summary.6 as u64,
                rank: summary.7.map(|rank| rank as u64),
                total_score: summary.8,
                aggregate_score: summary.9,
                telemetry_id: summary.10.map(|id| id as u32),
                validity: self
                    .get_onekv_candidate_validity_items(summary.0 as u32)
                    .await?,
                version: summary.11,
                location: summary.12,
                democracy_vote_count: summary.13 as u32,
                council_votes: summary.14,
                record_created_at: summary.15.timestamp_millis() as u64,
            }))
        } else {
            Ok(None)
        }
    }
}

impl PostgreSQLNetworkStorage {
    pub async fn save_onekv_nominator(
        &self,
        nominator: &OneKVNominator,
        history_record_count: i64,
    ) -> anyhow::Result<i32> {
        let account_id = AccountId::from_ss58_check(&nominator.address)?;
        let stash_account_id = AccountId::from_ss58_check(&nominator.stash_address)?;
        let proxy_account_id = AccountId::from_ss58_check(&nominator.proxy_address)?;
        self.save_account(&account_id).await?;
        self.save_account(&stash_account_id).await?;
        self.save_account(&proxy_account_id).await?;
        let nominator_save_result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_onekv_nominator (onekv_id, account_id, stash_account_id, proxy_account_id, bonded_amount, proxy_delay, last_nomination_at, nominator_created_at, average_stake, new_bonded_amount, reward_destination)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
        )
            .bind(&nominator.id)
            .bind(account_id.to_string())
            .bind(stash_account_id.to_string())
            .bind(proxy_account_id.to_string())
            .bind(nominator.bonded_amount.to_string())
            .bind(nominator.proxy_delay as i32)
            .bind(nominator.last_nomination_at as i64)
            .bind(nominator.created_at as i64)
            .bind(nominator.average_stake)
            .bind(nominator.new_bonded_amount)
            .bind(&nominator.reward_destination)
            .fetch_one(&self.connection_pool)
            .await?;
        // persist nominees
        let mut transaction = self.connection_pool.begin().await?;
        for nominee in &nominator.nominees {
            let stash_account_id = AccountId::from_ss58_check(&nominee.stash_address)?;
            self.save_account(&stash_account_id).await?;
            sqlx::query(
                r#"
                INSERT INTO sub_onekv_nominee (onekv_nominator_id, stash_account_id, name)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(nominator_save_result.0)
            .bind(stash_account_id.to_string())
            .bind(&nominee.name)
            .execute(&mut transaction)
            .await?;
        }
        transaction.commit().await?;

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

    pub async fn get_onekv_nominator_account_ids(&self) -> anyhow::Result<Vec<AccountId>> {
        let db_account_ids: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT stash_account_id
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
}
