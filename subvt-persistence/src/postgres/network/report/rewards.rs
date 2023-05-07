use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::report::ValidatorTotalReward;
use subvt_types::substrate::{Balance, Era};

impl PostgreSQLNetworkStorage {
    pub async fn get_validator_all_era_rewards(
        &self,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<Vec<(Era, Balance)>> {
        let era_rewards: Vec<(i64, i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT E.index, E.start_timestamp, E.end_timestamp, SUM(EV.amount::bigint)::bigint
            FROM sub_event_rewarded EV
            INNER JOIN sub_extrinsic_payout_stakers EX
                ON EV.block_hash = EX.block_hash
                AND EV.extrinsic_index = EX.extrinsic_index
                AND COALESCE(EV.nesting_index, '') = COALESCE(EX.nesting_index, '')
                AND EX.validator_account_id = EV.rewardee_account_id
            INNER JOIN sub_era E
                ON E.index = EX.era_index
            WHERE EV.rewardee_account_id = $1
            GROUP BY E.index
            ORDER BY E.index ASC;
            "#,
        )
        .bind(validator_account_id.to_string())
        .fetch_all(&self.connection_pool)
        .await?;
        let mut result = vec![];
        for era_reward in era_rewards {
            result.push((
                Era {
                    index: era_reward.0 as u32,
                    start_timestamp: era_reward.1 as u64,
                    end_timestamp: era_reward.1 as u64,
                },
                era_reward.3 as Balance,
            ))
        }
        Ok(result)
    }

    pub async fn get_validator_total_rewards(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> anyhow::Result<Vec<ValidatorTotalReward>> {
        let validator_total_rewards: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT ER.rewardee_account_id as validator_account_id, SUM(ER.amount::bigint)::bigint AS total_reward
            FROM sub_event_rewarded ER
            INNER JOIN sub_block B
                ON ER.block_hash = B.hash
                AND B.timestamp > $1
                AND B.timestamp < $2
            WHERE EXISTS (
                SELECT * FROM sub_era_validator EV
                WHERE EV.validator_account_id = ER.rewardee_account_id
            )
            GROUP BY ER.rewardee_account_id
            ORDER BY total_reward DESC;
            "#,
        )
        .bind(start_timestamp as i64)
        .bind(end_timestamp as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut result = vec![];
        for validator_total_reward in validator_total_rewards {
            result.push(ValidatorTotalReward {
                validator_account_id: AccountId::from_str(&validator_total_reward.0)?,
                total_reward: validator_total_reward.1 as Balance,
            })
        }
        Ok(result)
    }
}
