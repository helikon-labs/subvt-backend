use anyhow::Context;
use lazy_static::lazy_static;
use redis::{Client, RedisResult};
use subvt_config::Config;
use subvt_types::crypto::AccountId;
use subvt_types::report::BlockSummary;
use subvt_types::subvt::{NetworkStatus, ValidatorDetails};

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub struct Redis {
    client: Client,
}

impl Redis {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::open(CONFIG.redis.url.as_str())?;
        Ok(Redis { client })
    }
}

impl Redis {
    pub async fn set_finalized_block_summary(
        &self,
        block_summary: &BlockSummary,
    ) -> anyhow::Result<()> {
        let mut connection = self.client.get_async_connection().await?;
        redis::cmd("MSET")
            .arg(format!(
                "subvt:{}:validators:finalized_block_number",
                CONFIG.substrate.chain
            ))
            .arg(block_summary.number)
            .arg(format!(
                "subvt:{}:validators:finalized_block_hash",
                CONFIG.substrate.chain
            ))
            .arg(&block_summary.hash)
            .arg(format!(
                "subvt:{}:validators:finalized_block_timestamp",
                CONFIG.substrate.chain
            ))
            .arg(block_summary.timestamp)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn add_validator_account_id_to_active_set(
        &self,
        finalized_block_number: u64,
        account_id: &AccountId,
    ) -> anyhow::Result<()> {
        let mut connection = self.client.get_async_connection().await?;
        redis::cmd("SADD")
            .arg(format!(
                "subvt:{}:validators:{}:active:account_id_set",
                CONFIG.substrate.chain, finalized_block_number,
            ))
            .arg(account_id.to_string())
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn set_active_validator_details(
        &self,
        finalized_block_number: u64,
        validator_details: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        let mut connection = self.client.get_async_connection().await?;
        let validator_details_json = serde_json::to_string(validator_details)?;
        redis::cmd("SET")
            .arg(format!(
                "subvt:{}:validators:{}:active:validator:{}",
                CONFIG.substrate.chain, finalized_block_number, validator_details.account.id,
            ))
            .arg(validator_details_json)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn get_finalized_block_summary(&self) -> anyhow::Result<BlockSummary> {
        let number_key = format!(
            "subvt:{}:validators:finalized_block_number",
            CONFIG.substrate.chain
        );
        let hash_key = format!(
            "subvt:{}:validators:finalized_block_hash",
            CONFIG.substrate.chain
        );
        let timestamp_key = format!(
            "subvt:{}:validators:finalized_block_timestamp",
            CONFIG.substrate.chain
        );
        let mut connection = self.client.get_async_connection().await?;
        let (number, hash, timestamp) = redis::cmd("MGET")
            .arg(&[number_key, hash_key, timestamp_key])
            .query_async(&mut connection)
            .await?;
        Ok(BlockSummary {
            number,
            hash,
            timestamp,
        })
    }

    pub async fn validator_exists_by_account_id(
        &self,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let block_summary = self.get_finalized_block_summary().await?;
        let mut connection = self.client.get_async_connection().await?;
        let active_set_key = format!(
            "subvt:{}:validators:{}:active:account_id_set",
            CONFIG.substrate.chain, block_summary.number
        );
        let active_account_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(active_set_key)
            .query_async(&mut connection)
            .await?;
        let inactive_set_key = format!(
            "subvt:{}:validators:{}:inactive:account_id_set",
            CONFIG.substrate.chain, block_summary.number
        );
        let inactive_account_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(inactive_set_key)
            .query_async(&mut connection)
            .await?;
        Ok(active_account_ids.contains(&account_id.to_string())
            || inactive_account_ids.contains(&account_id.to_string()))
    }

    pub async fn fetch_validator_details(
        &self,
        finalized_block_number: u64,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<ValidatorDetails>> {
        if !self.validator_exists_by_account_id(account_id).await? {
            return Ok(None);
        }
        let mut connection = self.client.get_async_connection().await?;
        let active_validator_key = format!(
            "subvt:{}:validators:{}:active:validator:{}",
            CONFIG.substrate.chain, finalized_block_number, account_id,
        );
        let active_validator_json_string_result: RedisResult<String> = redis::cmd("GET")
            .arg(active_validator_key)
            .query_async(&mut connection)
            .await;
        let validator_json_string = match active_validator_json_string_result {
            Ok(validator_json_string) => validator_json_string,
            Err(_) => {
                let inactive_validator_key = format!(
                    "subvt:{}:validators:{}:inactive:validator:{}",
                    CONFIG.substrate.chain, finalized_block_number, account_id,
                );
                redis::cmd("GET")
                    .arg(inactive_validator_key)
                    .query_async(&mut connection)
                    .await?
            }
        };
        Ok(Some(serde_json::from_str(&validator_json_string)?))
    }

    pub async fn set_network_status(&self, network_status: &NetworkStatus) -> anyhow::Result<()> {
        let mut connection = self.client.get_async_connection().await?;
        let network_status_json = serde_json::to_string(network_status)?;
        redis::cmd("SET")
            .arg(format!("subvt:{}:network_status", CONFIG.substrate.chain))
            .arg(network_status_json)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn get_network_status(&self) -> anyhow::Result<NetworkStatus> {
        let mut connection = self.client.get_async_connection().await?;
        let key = format!("subvt:{}:network_status", CONFIG.substrate.chain);
        let status_json_string: String = redis::cmd("GET")
            .arg(key)
            .query_async(&mut connection)
            .await
            .context("Can't read network status from Redis.")?;
        let status: NetworkStatus = serde_json::from_str(&status_json_string)
            .context("Can't deserialize network status json.")?;
        Ok(status)
    }
}
