use anyhow::Context;
use redis::aio::Connection as RedisConnection;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use subvt_types::subvt::ValidatorDetails;

/// Does the initial population of the cached validator map.
pub(crate) async fn populate_validator_map(
    connection: &mut RedisConnection,
    prefix: &str,
    active_validator_account_ids: &HashSet<String>,
    all_validator_account_ids: &HashSet<String>,
    validator_map: &mut HashMap<String, ValidatorDetails>,
) -> anyhow::Result<()> {
    let all_keys: Vec<String> = all_validator_account_ids
        .iter()
        .map(|account_id| {
            format!(
                "{}:{}:validator:{}",
                prefix,
                if active_validator_account_ids.contains(account_id) {
                    "active"
                } else {
                    "inactive"
                },
                account_id
            )
        })
        .collect();
    let validator_json_strings: Vec<String> = redis::cmd("MGET")
        .arg(&all_keys)
        .query_async(connection)
        .await
        .context("Can't read validator JSON string from Redis.")?;
    log::debug!(
        "Got JSON string for {} validators.",
        validator_json_strings.len()
    );
    for validator_json_string in validator_json_strings {
        let validator: ValidatorDetails = serde_json::from_str(&validator_json_string).unwrap();
        validator_map.insert(validator.account.id.to_string(), validator);
    }
    log::info!(
        "Populated validator map with {} validators.",
        validator_map.len()
    );
    Ok(())
}
