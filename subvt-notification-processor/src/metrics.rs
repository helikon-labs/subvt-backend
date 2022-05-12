use once_cell::sync::Lazy;
use subvt_metrics::registry::{HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec};

const METRIC_PREFIX: &str = "subvt_notification_processor";

pub(crate) fn epoch_index(network_name: &str) -> IntGauge {
    static METER: Lazy<IntGaugeVec> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge_vec(
            METRIC_PREFIX,
            "epoch_index",
            "Last processed epoch index for epoch notifications",
            &["network_name"],
        )
        .unwrap()
    });
    METER.with_label_values(&[network_name])
}

pub(crate) fn era_index(network_name: &str) -> IntGauge {
    static METER: Lazy<IntGaugeVec> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge_vec(
            METRIC_PREFIX,
            "era_index",
            "Last processed epoch index for epoch notifications",
            &["network_name"],
        )
        .unwrap()
    });
    METER.with_label_values(&[network_name])
}

pub(crate) fn sent_notification_counter(notification_channel: &str) -> IntCounter {
    static METER: Lazy<IntCounterVec> = Lazy::new(|| {
        subvt_metrics::registry::register_int_counter_vec(
            METRIC_PREFIX,
            "sent_notification_count",
            "The number of notification successfully sent per notification channel",
            &["notification_channel"],
        )
        .unwrap()
    });
    METER.with_label_values(&[notification_channel])
}

pub(crate) fn channel_error_counter(notification_channel: &str) -> IntCounter {
    static METER: Lazy<IntCounterVec> = Lazy::new(|| {
        subvt_metrics::registry::register_int_counter_vec(
            METRIC_PREFIX,
            "channel_error_count",
            "The number of errors per notification channel",
            &["notification_channel"],
        )
        .unwrap()
    });
    METER.with_label_values(&[notification_channel])
}

fn notification_send_time_ms() -> HistogramVec {
    static METER: Lazy<HistogramVec> = Lazy::new(|| {
        subvt_metrics::registry::register_histogram_vec(
            METRIC_PREFIX,
            "notification_send_time_ms",
            "The time it takes to send a notification in milliseconds",
            &["notification_channel"],
            vec![
                50.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 1_500.0, 2_500.0, 5_000.0, 7_500.0,
                10_000.0, 15_000.0, 30_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}

pub(crate) fn observe_notification_send_time_ms(notification_channel: &str, elapsed_ms: f64) {
    match notification_send_time_ms().get_metric_with_label_values(&[notification_channel]) {
        Ok(metrics) => metrics.observe(elapsed_ms),
        Err(metrics_error) => {
            log::error!(
                "Cannot access notification send time metrics: {:?}",
                metrics_error
            );
        }
    }
}
