use crate::NotificationProcessor;
use chrono::{Datelike, Timelike, Utc};
use log::{error, info};
use subvt_types::app::NotificationPeriodType;
use tokio::runtime::Builder;

impl NotificationProcessor {
    /// Runs two cron-like jobs to process hourly and daily notifications.
    pub(crate) async fn start_hourly_and_daily_notification_processor(&self) -> anyhow::Result<()> {
        info!("Start hourly/daily notification processor.");
        let tokio_rt = Builder::new_current_thread().enable_all().build()?;
        let mut scheduler = job_scheduler::JobScheduler::new();
        // hourly jobs
        scheduler.add(job_scheduler::Job::new(
            "0 0/1 * * * *".parse().unwrap(),
            || {
                info!("Check for hourly notifications.");
                if let Err(error) = tokio_rt.block_on(
                    self.process_notifications(NotificationPeriodType::Hour, Utc::now().hour()),
                ) {
                    error!("Error while processing hourly notifications: {:?}", error);
                }
            },
        ));
        // daily jobs - send at midday UTC
        scheduler.add(job_scheduler::Job::new(
            "0 12 * * * *".parse().unwrap(),
            || {
                info!("Check for daily notifications.");
                if let Err(error) = tokio_rt.block_on(
                    self.process_notifications(NotificationPeriodType::Day, Utc::now().day()),
                ) {
                    error!("Error while processing daily notifications: {:?}", error);
                }
            },
        ));
        loop {
            scheduler.tick();
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
}
