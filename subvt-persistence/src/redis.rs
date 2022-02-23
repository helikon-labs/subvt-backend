use lazy_static::lazy_static;
use redis::{Client, RedisResult};
use subvt_config::Config;
use subvt_types::crypto::AccountId;
use subvt_types::subvt::ValidatorDetails;

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
    pub fn get_finalized_block_number(&self) -> anyhow::Result<u64> {
        let key = format!("subvt:{}:finalized_block_number", CONFIG.substrate.chain);
        let mut connection = self.client.get_connection()?;
        let finalized_block_number: u64 = redis::cmd("GET").arg(key).query(&mut connection)?;
        Ok(finalized_block_number)
    }

    pub fn validator_exists_by_account_id(&self, account_id: &AccountId) -> anyhow::Result<bool> {
        let finalized_block_number = self.get_finalized_block_number()?;
        let mut connection = self.client.get_connection()?;
        let active_set_key = format!(
            "subvt:{}:validators:{}:active:account_id_set",
            CONFIG.substrate.chain, finalized_block_number
        );
        let active_account_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(active_set_key)
            .query(&mut connection)?;
        let inactive_set_key = format!(
            "subvt:{}:validators:{}:inactive:account_id_set",
            CONFIG.substrate.chain, finalized_block_number
        );
        let inactive_account_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(inactive_set_key)
            .query(&mut connection)?;
        Ok(active_account_ids.contains(&account_id.to_string())
            || inactive_account_ids.contains(&account_id.to_string()))
    }

    pub fn fetch_validator_details(
        &self,
        account_id: &AccountId,
    ) -> anyhow::Result<ValidatorDetails> {
        let mut connection = self.client.get_connection()?;
        let finalized_block_number = self.get_finalized_block_number()?;
        let active_validator_key = format!(
            "subvt:{}:validators:{}:active:validator:{}",
            CONFIG.substrate.chain, finalized_block_number, account_id,
        );
        let active_validator_json_string_result: RedisResult<String> = redis::cmd("GET")
            .arg(active_validator_key)
            .query(&mut connection);
        let validator_json_string = match active_validator_json_string_result {
            Ok(validator_json_string) => validator_json_string,
            Err(_) => {
                let inactive_validator_key = format!(
                    "subvt:{}:validators:{}:inactive:validator:{}",
                    CONFIG.substrate.chain, finalized_block_number, account_id,
                );
                redis::cmd("GET")
                    .arg(inactive_validator_key)
                    .query(&mut connection)?
            }
        };
        Ok(serde_json::from_str(&validator_json_string)?)
    }
}
