use crate::query::QueryType;
use crate::TelegramBot;
use once_cell::sync::Lazy;
use subvt_metrics::registry::{IntCounter, IntCounterVec, IntGauge};

const METRIC_PREFIX: &str = "subvt_telegram_bot";

pub fn chat_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "chat_count",
            "Total number of chats",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn validator_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "validator_count",
            "Total number of validators",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn command_call_counter(command: &str) -> IntCounter {
    static METER: Lazy<IntCounterVec> = Lazy::new(|| {
        subvt_metrics::registry::register_int_counter_vec(
            METRIC_PREFIX,
            "command_call_count",
            "The number of calls per command",
            &["command"],
        )
        .unwrap()
    });
    METER.with_label_values(&[command])
}

pub fn query_call_counter(query: &QueryType) -> IntCounter {
    static METER: Lazy<IntCounterVec> = Lazy::new(|| {
        subvt_metrics::registry::register_int_counter_vec(
            METRIC_PREFIX,
            "query_call_count",
            "The number of calls per query",
            &["query"],
        )
        .unwrap()
    });
    METER.with_label_values(&[query.to_string().as_str()])
}

impl TelegramBot {
    pub(crate) async fn update_metrics_chat_count(&self) -> anyhow::Result<()> {
        chat_count().set(self.network_postgres.get_chat_count().await? as i64);
        Ok(())
    }

    pub(crate) async fn update_metrics_validator_count(&self) -> anyhow::Result<()> {
        validator_count().set(
            self.network_postgres
                .get_chat_total_validator_count()
                .await? as i64,
        );
        Ok(())
    }
}
