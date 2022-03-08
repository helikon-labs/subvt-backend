use once_cell::sync::Lazy;
use subvt_metrics::registry::IntGauge;

const METRIC_PREFIX: &str = "subvt_network_status_server";

pub fn target_best_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "target_best_block_number",
            "Number of the target finalized block on the node",
        )
            .unwrap()
    });
    METER.clone()
}

pub fn processed_best_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "processed_best_block_number",
            "Number of the last processed block",
        )
            .unwrap()
    });
    METER.clone()
}

pub fn subscription_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "subscription_count",
            "Number of subscribers to the service",
        )
            .unwrap()
    });
    METER.clone()
}