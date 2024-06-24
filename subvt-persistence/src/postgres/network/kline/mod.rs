use crate::postgres::network::PostgreSQLNetworkStorage;
use sqlx::types::BigDecimal;
use subvt_types::kline::KLine;

type DBKline = (
    i32,
    i64,
    String,
    String,
    BigDecimal,
    BigDecimal,
    BigDecimal,
    BigDecimal,
    BigDecimal,
    i64,
    BigDecimal,
    i32,
    BigDecimal,
    BigDecimal,
);

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

    pub async fn get_kline(
        &self,
        source_ticker: &str,
        target_ticker: &str,
        timestamp: u64,
    ) -> anyhow::Result<KLine> {
        let db_kline: DBKline = sqlx::query_as(
            r#"
            SELECT id, open_time, source_ticker, target_ticker, "open", high, low, "close", volume, close_time, quote_volume, "count", taker_buy_volume, taker_buy_quote_volume
            FROM sub_kline_historical
            WHERE source_ticker = $1 AND target_ticker= $2 AND open_time = $3
            "#,
        )
            .bind(source_ticker)
            .bind(target_ticker)
            .bind(timestamp as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(KLine {
            id: db_kline.0 as u32,
            open_time: db_kline.1 as u64,
            source_ticker: db_kline.2,
            target_ticker: db_kline.3,
            open: db_kline.4,
            high: db_kline.5,
            low: db_kline.6,
            close: db_kline.7,
            volume: db_kline.8,
            close_time: db_kline.9 as u64,
            quote_volume: db_kline.10,
            count: db_kline.11 as u32,
            taker_buy_volume: db_kline.12,
            taker_buy_quote_volume: db_kline.13,
        })
    }
}
