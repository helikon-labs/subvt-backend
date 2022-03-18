use once_cell::sync::Lazy;
use subvt_metrics::registry::{HistogramVec, IntCounter, IntCounterVec, IntGauge};

const METRIC_PREFIX: &str = "subvt_report_service";

pub(crate) fn request_counter() -> IntCounter {
    static METER: Lazy<IntCounter> = Lazy::new(|| {
        subvt_metrics::registry::register_int_counter(
            METRIC_PREFIX,
            "request_count",
            "The total number of requests made to the API",
        )
        .unwrap()
    });
    METER.clone()
}

pub(crate) fn connection_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "connection_count",
            "Number of API connections currently active",
        )
        .unwrap()
    });
    METER.clone()
}

fn response_time_ms() -> HistogramVec {
    static METER: Lazy<HistogramVec> = Lazy::new(|| {
        subvt_metrics::registry::register_histogram_vec(
            METRIC_PREFIX,
            "response_time_ms",
            "Response time in milliseconds",
            &["method", "status_code"],
            vec![
                50.0, 100.0, 250.0, 500.0, 750.0, 1_000.0, 1_500.0, 2_500.0, 5_000.0, 10_000.0,
                15_000.0, 30_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}

pub(crate) fn response_status_code_counter(status_code: &str) -> IntCounter {
    static METER: Lazy<IntCounterVec> = Lazy::new(|| {
        subvt_metrics::registry::register_int_counter_vec(
            METRIC_PREFIX,
            "response_status_code_count",
            "The number of response status codes",
            &["status_code"],
        )
        .unwrap()
    });
    METER.with_label_values(&[status_code])
}

pub(crate) fn observe_response_time_ms(method: &str, status_code: &str, elapsed_ms: f64) {
    match response_time_ms().get_metric_with_label_values(&[method, status_code]) {
        Ok(metrics) => metrics.observe(elapsed_ms),
        Err(metrics_error) => {
            log::error!("Cannot access response time metrics: {:?}", metrics_error)
        }
    }
}
