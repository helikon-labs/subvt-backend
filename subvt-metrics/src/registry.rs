use once_cell::sync::Lazy;
use prometheus::opts;
pub use prometheus::{
    core::Collector, proto, Counter, CounterVec, Error, Gauge, GaugeVec, Histogram, HistogramOpts,
    HistogramTimer, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Registry,
};
use std::sync::{Arc, RwLock};

static DEFAULT_REGISTRY: Lazy<Arc<RwLock<Registry>>> =
    Lazy::new(|| Arc::new(RwLock::new(Registry::default())));

pub(crate) fn get_default_registry() -> Registry {
    DEFAULT_REGISTRY.read().unwrap().clone()
}

fn register<C: Collector + 'static>(c: C) -> prometheus::Result<()> {
    get_default_registry().register(Box::new(c))
}

pub fn register_gauge(prefix: &str, name: &str, help: &str) -> prometheus::Result<Gauge> {
    let gauge = Gauge::new(format!("{}::{}", prefix, name), help)?;
    register(gauge.clone())?;
    Ok(gauge)
}

#[allow(clippy::disallowed_types)]
pub fn register_gauge_vec(
    prefix: &str,
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<GaugeVec> {
    let gauge = GaugeVec::new(opts!(format!("{}::{}", prefix, name), help), label_names)?;
    register(gauge.clone())?;
    Ok(gauge)
}

#[allow(clippy::disallowed_types)]
pub fn register_int_gauge_vec(
    prefix: &str,
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<IntGaugeVec> {
    let gauge = IntGaugeVec::new(opts!(format!("{}::{}", prefix, name), help), label_names)?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_int_counter(
    prefix: &str,
    name: &str,
    help: &str,
) -> prometheus::Result<IntCounter> {
    let gauge = IntCounter::new(format!("{}::{}", prefix, name), help)?;
    register(gauge.clone())?;
    Ok(gauge)
}

#[allow(clippy::disallowed_types)]
pub fn register_int_counter_vec(
    prefix: &str,
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<IntCounterVec> {
    let gauge = IntCounterVec::new(opts!(format!("{}::{}", prefix, name), help), label_names)?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_int_gauge(prefix: &str, name: &str, help: &str) -> prometheus::Result<IntGauge> {
    let gauge = IntGauge::new(format!("{}::{}", prefix, name), help)?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_histogram(
    prefix: &str,
    name: &str,
    help: &str,
    buckets: Vec<f64>,
) -> prometheus::Result<Histogram> {
    let histogram = Histogram::with_opts(
        HistogramOpts::new(format!("{}::{}", prefix, name), help).buckets(buckets),
    )?;
    register(histogram.clone())?;
    Ok(histogram)
}

pub fn register_histogram_vec(
    prefix: &str,
    name: &str,
    help: &str,
    label_names: &[&str],
    buckets: Vec<f64>,
) -> prometheus::Result<HistogramVec> {
    let gauge = HistogramVec::new(
        HistogramOpts::new(format!("{}::{}", prefix, name), help).buckets(buckets),
        label_names,
    )?;
    register(gauge.clone())?;
    Ok(gauge)
}
