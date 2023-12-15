use crate::postgres::network::PostgreSQLNetworkStorage;
use chrono::{NaiveDateTime, TimeZone};
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::governance::polkassembly::{ReferendumPost, ReferendumStatus};

type PostgresReferenda = (
    i32,
    String,
    String,
    i16,
    Option<String>,
    Option<String>,
    String,
    NaiveDateTime,
);

impl PostgreSQLNetworkStorage {
    pub async fn save_or_update_referendum(
        &self,
        referendum: &ReferendumPost,
    ) -> anyhow::Result<i32> {
        let referendum_save_result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_referendum (post_id, proposer_account_id, type, track_id, title, method, status, pa_created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(post_id) DO UPDATE
            SET type = EXCLUDED.type, track_id = EXCLUDED.track_id, title = EXCLUDED.title, method = EXCLUDED.method, status = EXCLUDED.status, pa_created_at = EXCLUDED.pa_created_at, updated_at = now()
            RETURNING post_id
            "#,
        )
            .bind(referendum.post_id as i32)
            .bind(referendum.proposer.to_string())
            .bind(&referendum.ty)
            .bind(referendum.track_no as i16)
            .bind(&referendum.maybe_title)
            .bind(&referendum.maybe_method)
            .bind(referendum.status.to_string())
            .bind(referendum.created_at)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(referendum_save_result.0)
    }

    pub async fn get_open_referenda(
        &self,
        track_id: Option<u16>,
    ) -> anyhow::Result<Vec<ReferendumPost>> {
        let db_referenda: Vec<PostgresReferenda> = if let Some(track_id) = track_id {
            sqlx::query_as(
                r#"
            SELECT post_id, proposer_account_id, type, track_id, title, method, status, pa_created_at
            FROM sub_referendum
            WHERE track_id = $1
            AND (status = 'Deciding' OR status = 'Submitted' OR status = 'DecisionDepositPlaced')
            ORDER BY track_id ASC
            "#,
            )
                .bind(track_id as i16)
                .fetch_all(&self.connection_pool)
                .await?
        } else {
            sqlx::query_as(
                r#"
            SELECT post_id, proposer_account_id, type, track_id, title, method, status, pa_created_at
            FROM sub_referendum
            WHERE (status = 'Deciding' OR status = 'Submitted' OR status = 'DecisionDepositPlaced')
            ORDER BY track_id ASC
            "#,
            )
                .fetch_all(&self.connection_pool)
                .await?
        };
        let mut referenda = vec![];
        for db_referendum in &db_referenda {
            referenda.push(ReferendumPost {
                post_id: db_referendum.0 as u32,
                track_no: db_referendum.3 as u16,
                proposer: AccountId::from_str(&db_referendum.1)?,
                maybe_title: db_referendum.4.clone(),
                maybe_method: db_referendum.5.clone(),
                status: ReferendumStatus::from_str(&db_referendum.6)?,
                created_at: chrono::Utc.from_utc_datetime(&db_referendum.7),
                ty: db_referendum.2.clone(),
            })
        }
        Ok(referenda)
    }
}
