//! 1KV-related storage - for Polkadot and Kusama.
use crate::postgres::network::PostgreSQLNetworkStorage;
use chrono::NaiveDateTime;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::onekv::{
    OneKVCandidate, OneKVCandidateSummary, OneKVNominator, OneKVNominatorSummary, OneKVValidity,
};

type PostgresCandidateSummary = (
    i32,
    i64,
    String,
    Option<i64>,
    i64,
    Option<i64>,
    i64,
    Option<f64>,
    Option<f64>,
    Option<String>,
    i64,
    Vec<String>,
    NaiveDateTime,
);

impl PostgreSQLNetworkStorage {
    pub async fn save_onekv_candidate(
        &self,
        candidate: &OneKVCandidate,
        history_record_count: i64,
    ) -> anyhow::Result<i32> {
        let validator_account_id = AccountId::from_str(&candidate.stash_address)?;
        let kusama_account_id = if !candidate.kusama_stash_address.is_empty() {
            let kusama_account_id = AccountId::from_str(&candidate.kusama_stash_address)?;
            Some(kusama_account_id)
        } else {
            None
        };
        self.save_account(&validator_account_id).await?;
        let candidate_save_result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_onekv_candidate (validator_account_id, kusama_account_id, discovered_at, inclusion, commission, is_active, unclaimed_eras, nominated_at, offline_accumulated, offline_since, name, location, rank, is_valid, fault_count, democracy_vote_count, democracy_votes, council_stake, council_votes, score_updated_at, score_total, score_aggregate, score_inclusion, score_discovered, score_nominated, score_rank, score_unclaimed, score_bonded, score_faults, score_offline, score_randomness, score_span_inclusion, score_location, score_council_stake, score_democracy, score_asn, score_country, score_nominator_stake, score_provider, score_region)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40)
            RETURNING id
            "#,
        )
            .bind(validator_account_id.to_string())
            .bind(kusama_account_id.map(|account_id| account_id.to_string()))
            .bind(candidate.discovered_at as i64)
            .bind(candidate.inclusion)
            .bind(candidate.commission)
            .bind(candidate.is_active)
            .bind(&candidate.unclaimed_eras.as_ref().map(|v| v.iter().map(|i| *i  as i64).collect::<Vec<i64>>()))
            .bind(candidate.nominated_at.map(|last_valid| last_valid as i64))
            .bind(candidate.offline_accumulated)
            .bind(candidate.offline_since as i64)
            .bind(&candidate.name)
            .bind(&candidate.location)
            .bind(candidate.rank)
            .bind(candidate.is_valid())
            .bind(candidate.fault_count)
            .bind(candidate.democracy_vote_count.unwrap_or_default() as i64)
            .bind(&candidate.democracy_votes.clone().unwrap_or_default().iter().map(|i| *i  as i64).collect::<Vec<i64>>())
            .bind(&candidate.council_stake.clone().unwrap_or_default())
            .bind(&candidate.council_votes.clone().unwrap_or_default())
            .bind(candidate.score.as_ref().map(|score| score.updated_at as i64))
            .bind(candidate.score.as_ref().map(|score| score.total))
            .bind(candidate.score.as_ref().map(|score| score.aggregate))
            .bind(candidate.score.as_ref().map(|score| score.inclusion))
            .bind(candidate.score.as_ref().map(|score| score.discovered))
            .bind(candidate.score.as_ref().map(|score| score.nominated))
            .bind(candidate.score.as_ref().map(|score| score.rank))
            .bind(candidate.score.as_ref().map(|score| score.unclaimed))
            .bind(candidate.score.as_ref().map(|score| score.bonded))
            .bind(candidate.score.as_ref().map(|score| score.faults))
            .bind(candidate.score.as_ref().map(|score| score.offline))
            .bind(candidate.score.as_ref().map(|score| score.randomness))
            .bind(candidate.score.as_ref().map(|score| score.span_inclusion))
            .bind(candidate.score.as_ref().map(|score| score.location))
            .bind(candidate.score.as_ref().map(|score| score.council_stake))
            .bind(candidate.score.as_ref().map(|score| score.democracy))
            .bind(candidate.score.as_ref().map(|score| score.asn))
            .bind(candidate.score.as_ref().map(|score| score.country))
            .bind(candidate.score.as_ref().map(|score| score.nominator_stake))
            .bind(candidate.score.as_ref().map(|score| score.provider))
            .bind(candidate.score.as_ref().map(|score| score.region))
            .fetch_one(&self.connection_pool)
            .await?;

        // persist validity records and rank events
        let mut transaction = self.connection_pool.begin().await?;
        for validity in &candidate.validity {
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
    pub async fn get_onekv_candidate_summary_by_account_id(
        &self,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<OneKVCandidateSummary>> {
        let maybe_candidate_summary: Option<PostgresCandidateSummary> = sqlx::query_as(
            r#"
            SELECT id, discovered_at, name, nominated_at, offline_since, rank, fault_count, score_total, score_aggregate, location, democracy_vote_count, council_votes, created_at
            FROM sub_onekv_candidate
            WHERE validator_account_id = $1
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
            .bind(account_id.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(summary) = maybe_candidate_summary {
            Ok(Some(OneKVCandidateSummary {
                record_id: summary.0 as u32,
                discovered_at: summary.1 as u64,
                name: summary.2,
                nominated_at: summary.3.map(|t| t as u64),
                offline_since: summary.4 as u64,
                rank: summary.5.map(|rank| rank as u64),
                fault_count: summary.6 as u64,
                total_score: summary.7,
                aggregate_score: summary.8,
                validity: self
                    .get_onekv_candidate_validity_items(summary.0 as u32)
                    .await?,
                location: summary.9,
                democracy_vote_count: summary.10 as u32,
                council_votes: summary.11,
                record_created_at: summary.12.timestamp_millis() as u64,
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
        let account_id = AccountId::from_str(&nominator.address)?;
        let stash_account_id = AccountId::from_str(&nominator.stash_address)?;
        let proxy_account_id = AccountId::from_str(&nominator.proxy_address)?;
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
            let stash_account_id = AccountId::from_str(&nominee.stash_address)?;
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

    pub async fn get_onekv_nominator_stash_account_ids(&self) -> anyhow::Result<Vec<AccountId>> {
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

    pub async fn get_onekv_nominator_summaries(
        &self,
    ) -> anyhow::Result<Vec<OneKVNominatorSummary>> {
        let account_ids = self.get_onekv_nominator_stash_account_ids().await?;
        let mut nominator_summaries = Vec::new();
        for account_id in &account_ids {
            let db_nominator_summary: (i32, String, String, String, i64) = sqlx::query_as(
                r#"
                SELECT id, onekv_id, stash_account_id, bonded_amount, last_nomination_at
                FROM sub_onekv_nominator
                WHERE stash_account_id = $1
                ORDER BY id DESC
                LIMIT 1
                "#,
            )
            .bind(account_id.to_string())
            .fetch_one(&self.connection_pool)
            .await?;
            nominator_summaries.push(OneKVNominatorSummary {
                id: db_nominator_summary.0 as u64,
                onekv_id: db_nominator_summary.1.clone(),
                stash_account_id: *account_id,
                stash_address: account_id.to_ss58_check(),
                bonded_amount: db_nominator_summary.3.parse()?,
                last_nomination_at: db_nominator_summary.4 as u64,
            });
        }
        Ok(nominator_summaries)
    }

    pub async fn is_onekv_nominator_account_id(
        &self,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM sub_onekv_nominator
            WHERE stash_account_id = $1
            "#,
        )
        .bind(account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count.0 > 0)
    }
}
