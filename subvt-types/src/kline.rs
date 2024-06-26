use num_traits::ToPrimitive;
pub use sqlx::types::BigDecimal;

#[derive(Clone, Debug)]
pub struct KLine {
    pub id: u32,
    pub open_time: u64,
    pub source_ticker: String,
    pub target_ticker: String,
    pub open: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub close: BigDecimal,
    pub volume: BigDecimal,
    pub close_time: u64,
    pub quote_volume: BigDecimal,
    pub count: u32,
    pub taker_buy_volume: BigDecimal,
    pub taker_buy_quote_volume: BigDecimal,
}

impl KLine {
    pub fn close_to_f64(&self) -> Option<f64> {
        self.close.to_f64()
    }
}
