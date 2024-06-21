use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::kline::KLine;

impl PostgreSQLNetworkStorage {
    pub async fn kline_exists(
        &self,
        timestamp: i64,
        source_ticker: &str,
        target_ticker: &str,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(id) FROM sub_kline_historical
            WHERE open_time = $1 AND source_ticker = $2 AND target_ticker = $3
            "#,
        )
        .bind(timestamp)
        .bind(source_ticker)
        .bind(target_ticker)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn save_kline(&self, kline: &KLine) -> anyhow::Result<i32> {
        let save_result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_kline_historical (open_time, source_ticker, target_ticker, "open", high, low, "close", volume, close_time, quote_volume, "count", taker_buy_volume, taker_buy_quote_volume)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (open_time, source_ticker, target_ticker)
            DO UPDATE SET taker_buy_quote_volume = EXCLUDED.taker_buy_quote_volume
            RETURNING id
            "#,
        )
            .bind(kline.open_time as i64)
            .bind(kline.source_ticker.as_str())
            .bind(kline.target_ticker.as_str())
            .bind(&kline.open)
            .bind(&kline.high)
            .bind(&kline.low)
            .bind(&kline.close)
            .bind(&kline.volume)
            .bind(kline.close_time as i64)
            .bind(&kline.quote_volume)
            .bind(kline.count as i32)
            .bind(&kline.taker_buy_volume)
            .bind(&kline.taker_buy_quote_volume)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(save_result.0)
    }

    pub async fn get_kline_count(&self) -> anyhow::Result<u64> {
        let record_count: (i64,) = sqlx::query_as("SELECT COUNT(id) FROM sub_kline_historical")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(record_count.0 as u64)
    }
}
