use crate::postgres::network::PostgreSQLNetworkStorage;
use parity_scale_codec::Encode;
use subvt_types::substrate::RewardDestination;
use subvt_types::{crypto::AccountId, substrate::Balance};

impl PostgreSQLNetworkStorage {
    pub async fn save_bond_extrinsic(
        &self,
        block_hash: &str,
        extrinsic_index: i32,
        is_nested_call: bool,
        batch_index: Option<String>,
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
            INSERT INTO sub_extrinsic_bond (block_hash, extrinsic_index, is_nested_call, batch_index, stash_account_id, controller_account_id, amount, reward_destination_encoded_hex, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT(block_hash, extrinsic_index, batch_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(is_nested_call)
            .bind(batch_index)
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
}
