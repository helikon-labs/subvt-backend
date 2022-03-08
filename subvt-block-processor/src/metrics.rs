use once_cell::sync::Lazy;
use subvt_metrics::registry::{Histogram, IntGauge};

const METRIC_PREFIX: &str = "subvt_block_processor";

pub fn processed_finalized_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "processed_finalized_block_number",
            "Number of the last processed block",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn target_finalized_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "target_finalized_block_number",
            "Number of the target finalized block on the node",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn block_processing_time_ms() -> Histogram {
    static METER: Lazy<Histogram> = Lazy::new(|| {
        subvt_metrics::registry::register_histogram(
            METRIC_PREFIX,
            "block_processing_time_ms",
            "Block processing time in milliseconds",
            vec![
                10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 250.0, 500.0, 750.0, 1_000.0, 1_500.0,
                2_000.0, 3_000.0, 4_000.0, 5_000.0, 7_500.0, 10_000.0, 15_000.0, 20_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}
