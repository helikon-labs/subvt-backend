use once_cell::sync::Lazy;
use subvt_metrics::registry::{Histogram, IntGauge};

const METRIC_PREFIX: &str = "subvt_onekv_updater";

pub fn last_run_timestamp_ms() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "last_run_timestamp_ms",
            "Timestamp (ms) for the last run",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn last_run_nominator_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "last_run_nominator_count",
            "Total number of 1KV nominators in the last run",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn last_run_candidate_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "last_run_candidate_count",
            "Total number of candidates in the last run",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn last_run_candidate_details_fetch_success_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "last_run_candidate_details_fetch_success_count",
            "Number of successful candidate details fetches in the last run",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn last_run_candidate_details_fetch_error_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "last_run_candidate_details_fetch_error_count",
            "Number of failed candidate details fetches in the last run",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn candidate_list_fetch_time_ms() -> Histogram {
    static METER: Lazy<Histogram> = Lazy::new(|| {
        subvt_metrics::registry::register_histogram(
            METRIC_PREFIX,
            "candidate_list_fetch_time_ms",
            "Histogram for candidate list fetch time in milliseconds",
            vec![
                100.0, 250.0, 500.0, 750.0, 1000.0, 1_500.0, 2_000.0, 3_000.0, 4_000.0, 5_000.0,
                7_500.0, 10_000.0, 15_000.0, 20_000.0, 30_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}

pub fn candidate_details_fetch_time_ms() -> Histogram {
    static METER: Lazy<Histogram> = Lazy::new(|| {
        subvt_metrics::registry::register_histogram(
            METRIC_PREFIX,
            "candidate_details_fetch_time_ms",
            "Histogram for candidate details fetch time in milliseconds",
            vec![
                100.0, 250.0, 500.0, 750.0, 1000.0, 1_500.0, 2_000.0, 3_000.0, 4_000.0, 5_000.0,
                7_500.0, 10_000.0, 15_000.0, 20_000.0, 30_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}

pub fn nominator_list_fetch_time_ms() -> Histogram {
    static METER: Lazy<Histogram> = Lazy::new(|| {
        subvt_metrics::registry::register_histogram(
            METRIC_PREFIX,
            "nominator_list_fetch_time_ms",
            "Histogram for nominator list fetch time in milliseconds",
            vec![
                100.0, 250.0, 500.0, 750.0, 1000.0, 1_500.0, 2_000.0, 3_000.0, 4_000.0, 5_000.0,
                7_500.0, 10_000.0, 15_000.0, 20_000.0, 30_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}
