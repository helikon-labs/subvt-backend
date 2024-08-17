use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::report::{Reward, ValidatorTotalReward};
use subvt_types::substrate::{Balance, Era};

impl PostgreSQLNetworkStorage {
    pub async fn get_validator_last_year_era_rewards(
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
            AND E.start_timestamp >= (EXTRACT(EPOCH FROM NOW() - INTERVAL '1 year')::bigint * 1000)
            GROUP BY E.index
            ORDER BY E.index ASC;
            "#,
        )
        .bind(validator_account_id.to_string())
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(era_rewards
            .iter()
            .map(|era_reward| {
                (
                    Era {
                        index: era_reward.0 as u32,
                        start_timestamp: era_reward.1 as u64,
                        end_timestamp: era_reward.1 as u64,
                    },
                    era_reward.3 as Balance,
                )
            })
            .collect())
    }

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
        Ok(era_rewards
            .iter()
            .map(|era_reward| {
                (
                    Era {
                        index: era_reward.0 as u32,
                        start_timestamp: era_reward.1 as u64,
                        end_timestamp: era_reward.1 as u64,
                    },
                    era_reward.3 as Balance,
                )
            })
            .collect())
    }

    pub async fn get_validator_total_rewards(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> anyhow::Result<Vec<ValidatorTotalReward>> {
        let validator_total_rewards: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT validator_account_id, SUM(amount::bigint)::bigint AS total_reward
            FROM sub_mat_view_validator_reward
            WHERE timestamp > $1
            AND timestamp < $2
            GROUP BY validator_account_id
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

    pub async fn get_rewards_in_time_range(
        &self,
        rewardee_account_id: &AccountId,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> anyhow::Result<Vec<Reward>> {
        let db_rewards: Vec<(i32, String, i64, i64, i32, i32, String)> = sqlx::query_as(
            r#"
            SELECT E.id, E.block_hash, B.number, B.timestamp, E.extrinsic_index, E.event_index, E.amount
            FROM sub_event_rewarded E
            INNER JOIN sub_block B ON B.hash = E.block_hash
            WHERE B.timestamp >= $1 AND B.timestamp < $2
            AND E.rewardee_account_id = $3
            ORDER BY B.timestamp ASC
            "#,
        )
            .bind(start_timestamp as i64)
            .bind(end_timestamp as i64)
            .bind(rewardee_account_id.to_string())
            .fetch_all(&self.connection_pool)
            .await?;
        let mut rewards = Vec::new();
        for db_reward in db_rewards.iter() {
            rewards.push(Reward {
                id: db_reward.0 as u32,
                block_hash: db_reward.1.clone(),
                block_number: db_reward.2 as u64,
                block_timestamp: db_reward.3 as u64,
                extrinsic_index: db_reward.4 as u32,
                event_index: db_reward.5 as u32,
                rewardee_account_id: *rewardee_account_id,
                amount: db_reward.6.parse()?,
            })
        }
        Ok(rewards)
    }
}
