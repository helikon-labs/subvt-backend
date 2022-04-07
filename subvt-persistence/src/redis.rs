use anyhow::Context;
use lazy_static::lazy_static;
use redis::{Client, RedisResult};
use subvt_config::Config;
use subvt_types::crypto::AccountId;
use subvt_types::subvt::{NetworkStatus, ValidatorDetails};

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub struct Redis {
    client: Client,
}

impl Redis {
    pub fn new() -> anyhow::Result<Self> {
        let client = redis::Client::open(CONFIG.redis.url.as_str())?;
        Ok(Redis { client })
    }
}

impl Redis {
    pub async fn get_finalized_block_number(&self) -> anyhow::Result<Option<u64>> {
        let key = format!(
            "subvt:{}:validators:finalized_block_number",
            CONFIG.substrate.chain
        );
        let mut connection = self.client.get_async_connection().await?;
        if let Ok(finalized_block_number) = redis::cmd("GET")
            .arg(key)
            .query_async(&mut connection)
            .await
        {
            Ok(Some(finalized_block_number))
        } else {
            Ok(None)
        }
    }

    pub async fn validator_exists_by_account_id(
        &self,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let finalized_block_number = if let Some(number) = self.get_finalized_block_number().await?
        {
            number
        } else {
            log::warn!("Finalized block number does not exist on Redis.");
            return Ok(false);
        };
        let mut connection = self.client.get_async_connection().await?;
        let active_set_key = format!(
            "subvt:{}:validators:{}:active:account_id_set",
            CONFIG.substrate.chain, finalized_block_number
        );
        let active_account_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(active_set_key)
            .query_async(&mut connection)
            .await?;
        let inactive_set_key = format!(
            "subvt:{}:validators:{}:inactive:account_id_set",
            CONFIG.substrate.chain, finalized_block_number
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
        account_id: &AccountId,
    ) -> anyhow::Result<Option<ValidatorDetails>> {
        if !self.validator_exists_by_account_id(account_id).await? {
            return Ok(None);
        }
        let mut connection = self.client.get_async_connection().await?;
        let finalized_block_number = if let Some(number) = self.get_finalized_block_number().await?
        {
            number
        } else {
            log::warn!("Finalized block number not found on Redis.");
            return Ok(None);
        };
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

    pub async fn get_current_network_status(&self) -> anyhow::Result<NetworkStatus> {
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
