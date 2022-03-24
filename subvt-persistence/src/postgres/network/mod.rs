//! Storage related to a network supported by SubVT.
//! Each supported network has a separate database.
use log::debug;
use parity_scale_codec::Encode;
use sqlx::{Pool, Postgres};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use subvt_config::Config;
use subvt_types::app::db::{
    PostgresBlock, PostgresPayoutStakersExtrinsic, PostgresSetControllerExtrinsic,
    PostgresValidateExtrinsic,
};
use subvt_types::app::event::{ChilledEvent, ValidatorOfflineEvent};
use subvt_types::app::extrinsic::{
    PayoutStakersExtrinsic, SetControllerExtrinsic, ValidateExtrinsic,
};
use subvt_types::app::Block;
use subvt_types::substrate::RewardDestination;
use subvt_types::{
    crypto::AccountId,
    rdb::ValidatorInfo,
    substrate::{
        argument::IdentificationTuple,
        EraStakers, ValidatorPreferences, ValidatorStake, {Balance, BlockHeader, Era},
    },
};

pub mod app_event;
pub mod democracy;
pub mod notify;
pub mod onekv;
pub mod report;
pub mod telegram;
pub mod telemetry;

type PostgresValidatorInfo = (
    Option<i64>,
    Option<i64>,
    i64,
    i64,
    i64,
    i64,
    i64,
    Option<String>,
    Option<i64>,
    Option<i64>,
    Option<bool>,
    Option<i32>,
    Option<String>,
    Option<i64>,
    Option<String>,
    Option<bool>,
);

pub struct PostgreSQLNetworkStorage {
    uri: String,
    connection_pool: Pool<Postgres>,
}

impl PostgreSQLNetworkStorage {
    pub async fn new(config: &Config, uri: String) -> anyhow::Result<PostgreSQLNetworkStorage> {
        debug!("Establishing network database connection pool...");
        let connection_pool = sqlx::postgres::PgPoolOptions::new()
            .connect_timeout(std::time::Duration::from_secs(
                config.network_postgres.connection_timeout_seconds,
            ))
            .max_connections(config.network_postgres.pool_max_connections)
            .connect(&uri)
            .await?;
        debug!("Network database connection pool established.");
        Ok(PostgreSQLNetworkStorage {
            uri,
            connection_pool,
        })
    }
}

impl PostgreSQLNetworkStorage {
    pub async fn save_account(&self, account_id: &AccountId) -> anyhow::Result<Option<AccountId>> {
        let maybe_result: Option<(String,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_account (id)
            VALUES ($1)
            ON CONFLICT (id) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(AccountId::from_str(&result.0)?))
        } else {
            Ok(None)
        }
    }

    pub async fn save_era(
        &self,
        era: &Era,
        total_stake: u128,
        era_stakers: &EraStakers,
    ) -> anyhow::Result<Option<i64>> {
        let nominator_count = {
            let mut nominator_account_id_set: HashSet<AccountId> = HashSet::new();
            for validator_stake in &era_stakers.stakers {
                for nominator_stake in &validator_stake.nominators {
                    nominator_account_id_set.insert(nominator_stake.account.id);
                }
            }
            nominator_account_id_set.len() as i64
        };
        let maybe_result: Option<(i64,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_era (index, start_timestamp, end_timestamp, active_nominator_count, total_stake, minimum_stake, maximum_stake, average_stake, median_stake)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (index) DO NOTHING
            RETURNING index
            "#,
        )
            .bind(era.index)
            .bind(era.start_timestamp as i64)
            .bind(era.end_timestamp as i64)
            .bind(nominator_count)
            .bind(total_stake.to_string())
            .bind(era_stakers.min_stake().1.to_string())
            .bind(era_stakers.max_stake().1.to_string())
            .bind(era_stakers.average_stake().to_string())
            .bind(era_stakers.median_stake().to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_epoch(&self, index: u64, era_index: u32) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_epoch (index, era_index)
            VALUES ($1, $2)
            ON CONFLICT (index) DO NOTHING
            RETURNING index
            "#,
        )
        .bind(index as i64)
        .bind(era_index)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_era_validators(
        &self,
        era_index: u32,
        active_validator_account_ids: &[AccountId],
        all_validator_account_ids: &[AccountId],
        bonded_account_id_map: &HashMap<AccountId, AccountId>,
        validator_stake_map: &HashMap<AccountId, ValidatorStake>,
        validator_prefs_map: &HashMap<AccountId, ValidatorPreferences>,
    ) -> anyhow::Result<()> {
        let mut transaction = self.connection_pool.begin().await?;
        for validator_account_id in all_validator_account_ids {
            // create validator account (if not exists)
            sqlx::query(
                r#"
                INSERT INTO sub_account (id)
                VALUES ($1)
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(validator_account_id.to_string())
            .execute(&mut transaction)
            .await?;
            // create controller account id (if not exists)
            let maybe_controller_account_id = bonded_account_id_map.get(validator_account_id);
            if let Some(controller_account_id) = maybe_controller_account_id {
                sqlx::query(
                    r#"
                    INSERT INTO sub_account (id)
                    VALUES ($1)
                    ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind(controller_account_id.to_string())
                .execute(&mut transaction)
                .await?;
            }
            let maybe_active_validator_index = active_validator_account_ids
                .iter()
                .position(|account_id| account_id == validator_account_id);
            // get prefs
            let maybe_validator_prefs = validator_prefs_map.get(validator_account_id);
            // get stakes for active
            let maybe_validator_stake = validator_stake_map.get(validator_account_id);

            // create record (if not exists)
            sqlx::query(
                r#"
                INSERT INTO sub_era_validator (era_index, validator_account_id, controller_account_id, is_active, active_validator_index, commission_per_billion, blocks_nominations, self_stake, total_stake, active_nominator_count)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (era_index, validator_account_id) DO NOTHING
                "#,
            )
                .bind(era_index)
                .bind(validator_account_id.to_string())
                .bind(maybe_controller_account_id.map(|id| id.to_string()))
                .bind(maybe_active_validator_index.is_some())
                .bind(maybe_active_validator_index.map(|index| index as i64))
                .bind(maybe_validator_prefs.map(|validator_prefs| validator_prefs.commission_per_billion))
                .bind(maybe_validator_prefs.map(|validator_prefs| validator_prefs.blocks_nominations))
                .bind(maybe_validator_stake.map(|validator_stake| validator_stake.self_stake.to_string()))
                .bind(maybe_validator_stake.map(|validator_stake| validator_stake.total_stake.to_string()))
                .bind(maybe_validator_stake.map(|validator_stake| validator_stake.nominators.len() as i32))
                .execute(&mut transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn save_era_stakers(&self, era_stakers: &EraStakers) -> anyhow::Result<()> {
        let mut transaction = self.connection_pool.begin().await?;
        for validator_stake in &era_stakers.stakers {
            sqlx::query(
                r#"
                INSERT INTO sub_account (id)
                VALUES ($1)
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(validator_stake.account.id.to_string())
            .execute(&mut transaction)
            .await?;
            for nominator_stake in &validator_stake.nominators {
                // create nominator account (if not exists)
                sqlx::query(
                    r#"
                    INSERT INTO sub_account (id)
                    VALUES ($1)
                    ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind(nominator_stake.account.id.to_string())
                .execute(&mut transaction)
                .await?;
                sqlx::query(
                    r#"
                    INSERT INTO sub_era_staker (era_index, validator_account_id, nominator_account_id, stake)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (era_index, validator_account_id, nominator_account_id) DO NOTHING
                    "#,
                )
                .bind(era_stakers.era.index)
                .bind(validator_stake.account.id.to_string())
                .bind(nominator_stake.account.id.to_string())
                .bind(nominator_stake.stake.to_string())
                .execute(&mut transaction)
                .await?;
            }
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn save_session_para_validators(
        &self,
        era_index: u32,
        session_index: u64,
        validator_account_ids: &Vec<&AccountId>,
    ) -> anyhow::Result<()> {
        let mut transaction = self.connection_pool.begin().await?;
        for validator_account_id in validator_account_ids {
            sqlx::query(
                r#"
                INSERT INTO sub_account (id)
                VALUES ($1)
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(validator_account_id.to_string())
            .execute(&mut transaction)
            .await?;
            sqlx::query(
                r#"
                    INSERT INTO sub_session_para_validator (era_index, session_index, validator_account_id)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (session_index, validator_account_id) DO NOTHING
                    "#,
            )
            .bind(era_index)
            .bind(session_index as i64)
            .bind(validator_account_id.to_string())
            .execute(&mut transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn era_exists(&self, era_index: u32) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(index) FROM sub_era
            WHERE index = $1
            "#,
        )
        .bind(era_index)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn update_era_reward_points(
        &self,
        era_index: u32,
        total_reward_points: u32,
    ) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64,)> = sqlx::query_as(
            r#"
            UPDATE sub_era SET total_reward_points = $1, updated_at = now()
            WHERE index = $2
            RETURNING index
            "#,
        )
        .bind(total_reward_points)
        .bind(era_index)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn update_era_total_validator_reward(
        &self,
        era_index: u32,
        total_validator_reward: Balance,
    ) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64,)> = sqlx::query_as(
            r#"
            UPDATE sub_era SET total_validator_reward = $1, updated_at = now()
            WHERE index = $2
            RETURNING index
            "#,
        )
        .bind(total_validator_reward.to_string())
        .bind(era_index)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn update_era_validator_reward_points(
        &self,
        era_index: u32,
        era_reward_points_map: HashMap<AccountId, u32>,
    ) -> anyhow::Result<()> {
        let mut transaction = self.connection_pool.begin().await?;
        for (validator_account_id, reward_points) in era_reward_points_map {
            sqlx::query(
                r#"
                UPDATE sub_era_validator SET reward_points = $1, updated_at = now()
                WHERE era_index = $2 AND validator_account_id = $3
                "#,
            )
            .bind(reward_points)
            .bind(era_index)
            .bind(validator_account_id.to_string())
            .execute(&mut transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn save_finalized_block(
        &self,
        block_hash: &str,
        block_header: &BlockHeader,
        block_timestamp: Option<u64>,
        maybe_author_account_id: Option<AccountId>,
        (era_index, epoch_index): (u32, u32),
        (metadata_version, runtime_version): (i16, i16),
    ) -> anyhow::Result<Option<String>> {
        let mut maybe_author_account_id_hex: Option<String> = None;
        if let Some(author_account_id) = maybe_author_account_id {
            maybe_author_account_id_hex = Some(author_account_id.to_string());
            self.save_account(&author_account_id).await?;
        }
        let maybe_result: Option<(String, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_block (hash, number, timestamp, author_account_id, era_index, epoch_index, parent_hash, state_root, extrinsics_root, is_finalized, metadata_version, runtime_version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (hash) DO NOTHING
            RETURNING hash
            "#)
            .bind(block_hash)
            .bind(block_header.get_number()? as u32)
            .bind(block_timestamp.map(|timestamp| timestamp as i64))
            .bind(maybe_author_account_id_hex)
            .bind(era_index)
            .bind(epoch_index)
            .bind(&block_header.parent_hash)
            .bind(&block_header.state_root)
            .bind(&block_header.extrinsics_root)
            .bind(true)
            .bind(metadata_version)
            .bind(runtime_version)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_validator_heartbeart_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        session_index: i64,
        im_online_key_hex_string: &str,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<()> {
        self.save_account(validator_account_id).await?;
        sqlx::query(
            r#"
            INSERT INTO sub_event_heartbeat_received (block_hash, extrinsic_index, event_index, session_index, im_online_key, validator_account_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(session_index)
            .bind(im_online_key_hex_string)
            .bind(validator_account_id.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn get_validator_offline_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<ValidatorOfflineEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, String)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, event_index, validator_account_id
            FROM sub_event_validator_offline
            WHERE block_hash = $1
            ORDER BY "id" ASC
            "#,
        )
        .bind(block_hash)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(ValidatorOfflineEvent {
                id: db_event.0 as u32,
                block_hash: db_event.1.clone(),
                event_index: db_event.2.map(|index| index as u32),
                validator_account_id: AccountId::from_str(&db_event.3)?,
            })
        }
        Ok(events)
    }

    pub async fn save_validators_offline_event(
        &self,
        block_hash: &str,
        event_index: i32,
        identification_tuples: &[IdentificationTuple],
    ) -> anyhow::Result<()> {
        for identification_tuple in identification_tuples {
            self.save_account(&identification_tuple.0).await?;
            sqlx::query(
                r#"
                INSERT INTO sub_event_validator_offline (block_hash, event_index, validator_account_id)
                VALUES ($1, $2, $3)
                "#,
            )
                .bind(block_hash)
                .bind(event_index)
                .bind(identification_tuple.0.to_string())
                .execute(&self.connection_pool)
                .await?;
        }
        Ok(())
    }

    pub async fn save_nomination(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        is_successful: bool,
        controller_account_id: &AccountId,
        validator_account_ids: &[AccountId],
    ) -> anyhow::Result<()> {
        self.save_account(controller_account_id).await?;
        let maybe_extrinsic_nominate_id: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_nominate (block_hash, extrinsic_index, is_nested_call, controller_account_id, is_successful)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(controller_account_id.to_string())
            .bind(is_successful)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(extrinsic_nominate_id) = maybe_extrinsic_nominate_id {
            for validator_account_id in validator_account_ids {
                self.save_account(validator_account_id).await?;
                sqlx::query(
                    r#"
                    INSERT INTO sub_extrinsic_nominate_validator (extrinsic_nominate_id, validator_account_id)
                    VALUES ($1, $2)
                    ON CONFLICT (extrinsic_nominate_id, validator_account_id) DO NOTHING
                    "#)
                    .bind(extrinsic_nominate_id.0)
                    .bind(validator_account_id.to_string())
                    .execute(&self.connection_pool)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn get_chilled_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<ChilledEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, i32, String)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, event_index, stash_account_id
            FROM sub_event_chilled
            WHERE block_hash = $1
            ORDER BY "id" ASC
            "#,
        )
        .bind(block_hash)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(ChilledEvent {
                id: db_event.0 as u32,
                block_hash: db_event.1.clone(),
                extrinsic_index: db_event.2.map(|index| index as u32),
                event_index: db_event.3 as u32,
                stash_account_id: AccountId::from_str(&db_event.4)?,
            })
        }
        Ok(events)
    }

    pub async fn save_chilled_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        stash_account_id: &AccountId,
    ) -> anyhow::Result<()> {
        self.save_account(stash_account_id).await?;
        sqlx::query(
            r#"
            INSERT INTO sub_event_chilled (block_hash, extrinsic_index, event_index, stash_account_id)
            VALUES ($1, $2, $3, $4)
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(stash_account_id.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn save_era_paid_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        era_index: u32,
        validator_payout: Balance,
        remainder: Balance,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_event_era_paid (block_hash, extrinsic_index, event_index, era_index, validator_payout, remainder)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#)
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(era_index)
            .bind(validator_payout.to_string())
            .bind(remainder.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn save_nominator_kicked_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        validator_account_id: &AccountId,
        nominator_account_id: &AccountId,
    ) -> anyhow::Result<()> {
        self.save_account(validator_account_id).await?;
        self.save_account(nominator_account_id).await?;
        sqlx::query(
            r#"
            INSERT INTO sub_event_nominator_kicked (block_hash, extrinsic_index, event_index, validator_account_id, nominator_account_id)
            VALUES ($1, $2, $3, $4, $5)
            "#)
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(validator_account_id.to_string())
            .bind(nominator_account_id.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn save_rewarded_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        rewardee_account_id: &AccountId,
        amount: Balance,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(rewardee_account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_rewarded (block_hash, extrinsic_index, event_index, rewardee_account_id, amount)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(rewardee_account_id.to_string())
            .bind(amount.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_slashed_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        validator_account_id: &AccountId,
        amount: Balance,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(validator_account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_slashed (block_hash, extrinsic_index, event_index, validator_account_id, amount)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(validator_account_id.to_string())
            .bind(amount.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_new_account_event(
        &self,
        block_hash: &str,
        block_number: u64,
        block_timestamp: Option<u64>,
        extrinsic_index: Option<i32>,
        event_index: i32,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<String>> {
        self.save_account(account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_new_account (block_hash, extrinsic_index, event_index, account_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(block_hash)
        .bind(extrinsic_index)
        .bind(event_index)
        .bind(account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        if maybe_result.is_none() {
            return Ok(None);
        }
        // update account
        let maybe_result: Option<(String,)> = sqlx::query_as(
            r#"
            UPDATE sub_account SET discovered_at_block_hash = $1, discovered_at_block_number = $2, discovered_at = $3, updated_at = now()
            WHERE id = $4
            RETURNING id
            "#,
        )
        .bind(block_hash)
        .bind(block_number as i64)
        .bind(block_timestamp.map(|timestamp| timestamp as i64))
        .bind(account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_killed_account_event(
        &self,
        block_hash: &str,
        block_number: u64,
        block_timestamp: Option<u64>,
        extrinsic_index: Option<i32>,
        event_index: i32,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<String>> {
        self.save_account(account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_killed_account (block_hash, extrinsic_index, event_index, account_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(account_id.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if maybe_result.is_none() {
            return Ok(None);
        }
        // update account
        let maybe_result: Option<(String,)> = sqlx::query_as(
            r#"
            UPDATE sub_account SET killed_at_block_hash = $1, discovered_at_block_number = $2, discovered_at = $3, updated_at = now()
            WHERE id = $4
            RETURNING id
            "#,
        )
        .bind(block_hash)
        .bind(block_number as i64)
        .bind(block_timestamp.map(|timestamp| timestamp as i64))
        .bind(account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn get_block_hash(&self, block_number: u64) -> anyhow::Result<Option<String>> {
        Ok(sqlx::query_as(
            r#"
            SELECT hash FROM sub_block
            WHERE "number" = $1
            "#,
        )
        .bind(block_number as i64)
        .fetch_optional(&self.connection_pool)
        .await?
        .map(|hash: (String,)| hash.0))
    }

    pub async fn get_block_by_number(&self, block_number: u64) -> anyhow::Result<Option<Block>> {
        let maybe_db_block: Option<PostgresBlock> = sqlx::query_as(
            r#"
            SELECT hash, number, timestamp, author_account_id, era_index, epoch_index, is_finalized, metadata_version, runtime_version
            FROM sub_block
            WHERE "number" = $1
            "#,
        )
            .bind(block_number as i64)
            .fetch_optional(&self.connection_pool)
            .await?;
        match maybe_db_block {
            Some(db_block) => Ok(Some(Block::from(db_block)?)),
            None => Ok(None),
        }
    }

    pub async fn get_processed_block_height(&self) -> anyhow::Result<u64> {
        let processed_block_height: (i64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(MAX(number), 0) from sub_block
            "#,
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(processed_block_height.0 as u64)
    }

    pub async fn save_batch_item_completed_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_event_batch_item_completed (block_hash, extrinsic_index, event_index)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(block_hash)
        .bind(extrinsic_index)
        .bind(event_index)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn save_batch_interrupted_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        item_index: i32,
        dispatch_error_debug: String,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_event_batch_interrupted (block_hash, extrinsic_index, event_index, item_index, dispatch_error_debug)
            VALUES ($1, $2, $3, $4, $5)
            "#)
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(item_index)
            .bind(dispatch_error_debug)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn save_batch_completed_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_event_batch_completed (block_hash, extrinsic_index, event_index)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(block_hash)
        .bind(extrinsic_index)
        .bind(event_index)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn get_validate_extrinsics_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<ValidateExtrinsic>> {
        let db_extrinsics: Vec<PostgresValidateExtrinsic> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, is_nested_call, stash_account_id, controller_account_id, commission_per_billion, blocks_nominations, is_successful
            FROM sub_extrinsic_validate
            WHERE block_hash = $1 AND is_successful = true
            ORDER BY "id" ASC
            "#,
        )
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut extrinsics = Vec::new();
        for db_extrinsic in db_extrinsics {
            extrinsics.push(ValidateExtrinsic::from(db_extrinsic)?)
        }
        Ok(extrinsics)
    }

    pub async fn save_validate_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        is_successful: bool,
        (stash_account_id, controller_account_id): (&AccountId, &AccountId),
        validator_preferences: &ValidatorPreferences,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(stash_account_id).await?;
        self.save_account(controller_account_id).await?;
        let maybe_result: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_validate (block_hash, extrinsic_index, is_nested_call, stash_account_id, controller_account_id, commission_per_billion, blocks_nominations, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(stash_account_id.to_string())
            .bind(controller_account_id.to_string())
            .bind(validator_preferences.commission_per_billion)
            .bind(validator_preferences.blocks_nominations)
            .bind(is_successful)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_payout_stakers_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        is_successful: bool,
        (caller_account_id, validator_account_id): (&AccountId, &AccountId),
        era_index: u32,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(caller_account_id).await?;
        self.save_account(validator_account_id).await?;
        let maybe_result: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_payout_stakers (block_hash, extrinsic_index, is_nested_call, caller_account_id, validator_account_id, era_index, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(caller_account_id.to_string())
            .bind(validator_account_id.to_string())
            .bind(era_index)
            .bind(is_successful)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn get_payout_stakers_extrinsics_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<PayoutStakersExtrinsic>> {
        let db_extrinsics: Vec<PostgresPayoutStakersExtrinsic> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, is_nested_call, caller_account_id, validator_account_id, era_index, is_successful
            FROM sub_extrinsic_payout_stakers
            WHERE block_hash = $1 AND is_successful = true
            ORDER BY "id" ASC
            "#,
        )
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut extrinsics = Vec::new();
        for db_extrinsic in db_extrinsics {
            extrinsics.push(PayoutStakersExtrinsic::from(db_extrinsic)?)
        }
        Ok(extrinsics)
    }

    pub async fn save_set_controller_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        is_successful: bool,
        caller_account_id: &AccountId,
        controller_account_id: &AccountId,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(caller_account_id).await?;
        self.save_account(controller_account_id).await?;
        let maybe_result: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_set_controller (block_hash, extrinsic_index, is_nested_call, caller_account_id, controller_account_id, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(caller_account_id.to_string())
            .bind(controller_account_id.to_string())
            .bind(is_successful)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn get_set_controller_extrinsics_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<SetControllerExtrinsic>> {
        let db_extrinsics: Vec<PostgresSetControllerExtrinsic> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, is_nested_call, caller_account_id, controller_account_id, is_successful
            FROM sub_extrinsic_set_controller
            WHERE block_hash = $1 AND is_successful = true
            ORDER BY "id" ASC
            "#,
        )
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut extrinsics = Vec::new();
        for db_extrinsic in db_extrinsics {
            extrinsics.push(SetControllerExtrinsic::from(db_extrinsic)?)
        }
        Ok(extrinsics)
    }

    pub async fn save_bond_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        is_successful: bool,
        (stash_account_id, controller_account_id, amount, reward_destination): (
            &AccountId,
            &AccountId,
            Balance,
            &RewardDestination,
        ),
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(stash_account_id).await?;
        self.save_account(controller_account_id).await?;
        let maybe_result: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_bond (block_hash, extrinsic_index, is_nested_call, stash_account_id, controller_account_id, amount, reward_destination_encoded_hex, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(stash_account_id.to_string())
            .bind(controller_account_id.to_string())
            .bind(amount.to_string())
            .bind(format!("0x{}", hex::encode_upper(reward_destination.encode())))
            .bind(is_successful)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn get_validator_info(
        &self,
        block_hash: &str,
        validator_account_id: &AccountId,
        is_active: bool,
        era_index: u32,
    ) -> anyhow::Result<ValidatorInfo> {
        let validator_info: PostgresValidatorInfo = sqlx::query_as(
            r#"
            SELECT discovered_at, killed_at, slash_count, offline_offence_count, active_era_count, inactive_era_count, total_reward_points, unclaimed_eras, blocks_authored, reward_points, heartbeat_received, onekv_candidate_record_id, onekv_binary_version, onekv_rank, onekv_location, onekv_is_valid
            FROM sub_get_validator_info($1, $2, $3, $4)
            "#
        )
            .bind(block_hash)
            .bind(validator_account_id.to_string())
            .bind(is_active)
            .bind(era_index as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        let mut unclaimed_era_indices: Vec<u32> = Vec::new();
        if let Some(concated_string) = validator_info.7 {
            for unclaimed_era_index_string in concated_string.split(',') {
                if let Ok(unclaimed_era_index) = unclaimed_era_index_string.parse::<u32>() {
                    unclaimed_era_indices.push(unclaimed_era_index);
                }
            }
        }
        Ok(ValidatorInfo {
            discovered_at: validator_info.0.map(|value| value as u64),
            killed_at: validator_info.1.map(|value| value as u64),
            slash_count: validator_info.2 as u64,
            offline_offence_count: validator_info.3 as u64,
            active_era_count: validator_info.4 as u64,
            inactive_era_count: validator_info.5 as u64,
            total_reward_points: validator_info.6 as u64,
            unclaimed_era_indices,
            blocks_authored: validator_info.8.map(|value| value as u64),
            reward_points: validator_info.9.map(|value| value as u64),
            heartbeat_received: validator_info.10,
            onekv_candidate_record_id: validator_info.11.map(|value| value as u32),
            onekv_binary_version: validator_info.12,
            onekv_rank: validator_info.13.map(|value| value as u64),
            onekv_location: validator_info.14,
            onekv_is_valid: validator_info.15,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save_heartbeat_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        is_successful: bool,
        block_number: u32,
        session_index: u32,
        validator_index: u32,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_extrinsic_heartbeat (block_hash, extrinsic_index, is_nested_call, block_number, session_index, validator_index, validator_account_id, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(block_number as i64)
            .bind(session_index as i64)
            .bind(validator_index as i64)
            .bind(validator_account_id.to_string())
            .bind(is_successful)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }
}
