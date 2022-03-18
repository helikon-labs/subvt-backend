use once_cell::sync::Lazy;
use subvt_metrics::registry::{Histogram, IntGauge};

const METRIC_PREFIX: &str = "subvt_network_status_updater";

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

pub fn processing_time_ms() -> Histogram {
    static METER: Lazy<Histogram> = Lazy::new(|| {
        subvt_metrics::registry::register_histogram(
            METRIC_PREFIX,
            "block_processing_time_ms",
            "Block processing time in milliseconds",
            vec![
                100.0, 250.0, 500.0, 750.0, 1_000.0, 1_250.0, 1_500.0, 1_750.0, 2_000.0, 3_500.0,
                5_000.0, 7_500.0, 10_000.0, 15_000.0, 20_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}
