use crate::postgres::network::PostgreSQLNetworkStorage;
use std::collections::HashMap;
use subvt_types::report::{ParaVote, ParaVoteType, SessionParaVoteReport};

impl PostgreSQLNetworkStorage {
    pub async fn get_session_para_validator_votes(
        &self,
        session_index: u64,
        para_validator_index: u64,
    ) -> anyhow::Result<Vec<ParaVote>> {
        let para_votes: Vec<(i64, String, i64, Option<bool>)> = sqlx::query_as(
            r#"
            SELECT B.number, PV.block_hash, PV.para_id, PV.is_explicit
            FROM sub_para_vote PV
            INNER JOIN sub_block B
                ON B.hash = PV.block_hash
            WHERE PV.session_index = $1
            AND PV.para_validator_index = $2
            ORDER BY B.number ASC
            "#,
        )
        .bind(session_index as i64)
        .bind(para_validator_index as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(para_votes
            .iter()
            .map(|vote| ParaVote {
                block_number: vote.0 as u64,
                block_hash: vote.1.clone(),
                session_index,
                para_id: vote.2 as u64,
                para_validator_index,
                vote: match vote.3 {
                    Some(true) => ParaVoteType::EXPLICIT,
                    Some(false) => ParaVoteType::IMPLICIT,
                    None => ParaVoteType::MISSED,
                },
            })
            .collect())
    }

    pub async fn get_session_para_vote_summaries(
        &self,
        session_index: u64,
    ) -> anyhow::Result<Vec<SessionParaVoteReport>> {
        let para_vote_counts: Vec<(i64, Option<bool>, i64)> = sqlx::query_as(
            r#"
            SELECT para_id, is_explicit, COUNT(DISTINCT id) AS vote_count
            FROM sub_para_vote
            WHERE session_index = $1
            GROUP BY para_id, is_explicit
            ORDER BY para_id ASC, is_explicit ASC
            "#,
        )
        .bind(session_index as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut report_map: HashMap<u64, SessionParaVoteReport> = HashMap::new();
        for para_vote_count in &para_vote_counts {
            let para_id = para_vote_count.0 as u64;
            let report = report_map.entry(para_id).or_insert(SessionParaVoteReport {
                para_id,
                para_votes_summary: Default::default(),
            });
            match para_vote_count.1 {
                Some(false) => report.para_votes_summary.implicit = para_vote_count.2 as u32,
                Some(true) => report.para_votes_summary.explicit = para_vote_count.2 as u32,
                None => report.para_votes_summary.missed = para_vote_count.2 as u32,
            }
        }
        let mut reports: Vec<SessionParaVoteReport> =
            report_map.iter().map(|pair| pair.1.clone()).collect();
        reports.sort_by_key(|vec| vec.para_id);
        Ok(reports)
    }
}
