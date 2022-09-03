//! Telemetry-related storage. Used by the `subvt-telemetry-processor` crate, and other crates
//! that query the telemetry data (validator details, notification generator, etc.).
use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::telemetry::{NodeDetails, NodeHardware, NodeLocation, NodeStats};

impl PostgreSQLNetworkStorage {
    pub async fn update_node_best_block(
        &self,
        node_id: u64,
        best_block_number: u64,
        best_block_hash: &str,
    ) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64,)> = sqlx::query_as(
            r#"
            UPDATE sub_telemetry_node SET best_block_number = $1, best_block_hash = $2, updated_at = now()
            WHERE id = $3
            RETURNING id
            "#,
        )
            .bind(best_block_number as i64)
            .bind(best_block_hash)
            .bind(node_id as i64)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn update_node_finalized_block(
        &self,
        node_id: u64,
        finalized_block_number: u64,
        finalized_block_hash: &str,
    ) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64,)> = sqlx::query_as(
            r#"
            UPDATE sub_telemetry_node SET finalized_block_number = $1, finalized_block_hash = $2, updated_at = now()
            WHERE id = $3
            RETURNING id
            "#,
        )
            .bind(finalized_block_number as i64)
            .bind(finalized_block_hash)
            .bind(node_id as i64)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn update_node_location(
        &self,
        node_id: u64,
        location: &NodeLocation,
    ) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64,)> = sqlx::query_as(
            r#"
            UPDATE sub_telemetry_node SET location = $1, latitude = $2, longitude = $3, updated_at = now()
            WHERE id = $4
            RETURNING id
            "#,
        )
            .bind(&location.2)
            .bind(location.0 as f64)
            .bind(location.1 as f64)
            .bind(node_id as i64)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_node(
        &self,
        node_id: u64,
        node_details: &NodeDetails,
        startup_time: Option<u64>,
        location: &Option<NodeLocation>,
    ) -> anyhow::Result<()> {
        let account_id_str = if let Some(address) = &node_details.controller_address {
            if let Ok(account_id) = AccountId::from_str(address) {
                Some(account_id.to_string())
            } else {
                None
            }
        } else {
            None
        };
        sqlx::query(
            r#"
            INSERT INTO sub_telemetry_node (id, controller_account_id, name, client_implementation, client_version, startup_time, location, latitude, longitude)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT(id) DO UPDATE
            SET controller_account_id = EXCLUDED.controller_account_id, name = EXCLUDED.name, client_implementation = EXCLUDED.client_implementation, client_version = EXCLUDED.client_version, startup_time = EXCLUDED.startup_time,  location = EXCLUDED.location, latitude = EXCLUDED.latitude, longitude = EXCLUDED.longitude
            "#,
        )
            .bind(node_id as i64)
            .bind(account_id_str)
            .bind(&node_details.name)
            .bind(&node_details.implementation)
            .bind(&node_details.version)
            .bind(startup_time.map(|startup_time| startup_time as i64))
            .bind(location.as_ref().map(|location| location.2.clone()))
            .bind(location.as_ref().map(|location| location.0 as f64))
            .bind(location.as_ref().map(|location| location.1 as f64))
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn save_node_stats(&self, node_id: u64, stats: &NodeStats) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_telemetry_node_stats (node_id, peer_count, queued_tx_count)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(node_id as i64)
        .bind(stats.peer_count as i32)
        .bind(stats.queued_tx_count as i32)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn remove_node(&self, node_id: u64) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64,)> =
            sqlx::query_as("DELETE FROM sub_telemetry_node WHERE id = $1 RETURNING id")
                .bind(node_id as i64)
                .fetch_optional(&self.connection_pool)
                .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_node_network_stats(
        &self,
        node_id: u64,
        data: &NodeHardware,
    ) -> anyhow::Result<()> {
        let mut transaction = self.connection_pool.begin().await?;
        for i in 0..data.0.len() {
            let date_time = chrono::NaiveDateTime::from_timestamp(
                data.2[i] as i64 / 1000,
                (data.2[i] as i64 % 1000) as u32,
            );
            sqlx::query(
                r#"
                INSERT INTO sub_telemetry_node_network_stats (time, node_id, download_bandwidth, upload_bandwidth)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT(time, node_id) DO UPDATE
                SET download_bandwidth = EXCLUDED.download_bandwidth, upload_bandwidth = EXCLUDED.upload_bandwidth
                "#,
            )
                .bind(date_time)
                .bind(node_id as i64)
                .bind(data.1[i] as f64)
                .bind(data.0[i] as f64)
                .execute(&mut transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn update_best_block_number(
        &self,
        best_block_number: u64,
        best_block_timestamp: u64,
        average_block_time_ms: Option<u64>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_telemetry_network_status
            SET best_block_number = $1, best_block_timestamp = $2, average_block_time_ms = $3
            WHERE id = 1
            "#,
        )
        .bind(best_block_number as i64)
        .bind(best_block_timestamp as i64)
        .bind(average_block_time_ms.map(|ms| ms as i64))
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn update_finalized_block_number(
        &self,
        finalized_block_number: u64,
        finalized_block_hash: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_telemetry_network_status
            SET finalized_block_number = $1, finalized_block_hash = $2, updated_at = now()
            WHERE id = 1
            "#,
        )
        .bind(finalized_block_number as i64)
        .bind(finalized_block_hash)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}
