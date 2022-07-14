use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::report::{ParaVote, ParaVoteType};

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
}
