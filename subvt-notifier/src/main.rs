//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_notifier::Notifier;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: Notifier = Notifier::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}
