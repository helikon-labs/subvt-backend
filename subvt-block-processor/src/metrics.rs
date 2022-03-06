use once_cell::sync::Lazy;
use subvt_metrics::registry::{Histogram, IntGauge};

pub fn processed_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            "subvt_block_processor::last_processed_block_number",
            "Number of the last processed block",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn target_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            "subvt_block_processor::target_block_number",
            "Number of the target finalized block on the node",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn block_processing_time_ms() -> Histogram {
    static METER: Lazy<Histogram> = Lazy::new(|| {
        subvt_metrics::registry::register_histogram(
            "subvt_block_processor::block_processing_time_ms",
            "Block processing time in milliseconds",
            vec![
                10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 250.0, 500.0, 750.0, 1000.0, 1500.0,
                2000.0, 3000.0, 4000.0, 5000.0, 7500.0, 10000.0, 15000.0, 20000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}
