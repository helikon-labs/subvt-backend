use crate::NotificationProcessor;
use chrono::{Datelike, Timelike, Utc};
use subvt_types::app::NotificationPeriodType;
use tokio::runtime::Builder;

impl NotificationProcessor {
    /// Runs two cron-like jobs to process hourly and daily notifications.
    pub(crate) fn start_hourly_and_daily_notification_processor(
        &'static self,
    ) -> anyhow::Result<()> {
        log::info!("Start hourly & daily notification processor.");
        let tokio_rt = Builder::new_multi_thread().enable_all().build().unwrap();
        std::thread::spawn(move || {
            let mut scheduler = job_scheduler::JobScheduler::new();
            // hourly jobs
            scheduler.add(job_scheduler::Job::new(
                "0 0 0/1 * * *".parse().unwrap(),
                || {
                    log::info!("New hour: check for notifications.");
                    tokio_rt.spawn(async {
                        if let Err(error) = self
                            .process_notifications(
                                None,
                                NotificationPeriodType::Hour,
                                Utc::now().hour(),
                            )
                            .await
                        {
                            log::error!("Error while processing hourly notifications: {:?}", error);
                        }
                    });
                },
            ));
            // daily jobs - send at midday UTC
            scheduler.add(job_scheduler::Job::new(
                "0 0 12 * * *".parse().unwrap(),
                || {
                    log::info!("New day: check for notifications.");
                    tokio_rt.spawn(async {
                        if let Err(error) = self
                            .process_notifications(
                                None,
                                NotificationPeriodType::Day,
                                Utc::now().day(),
                            )
                            .await
                        {
                            log::error!("Error while processing daily notifications: {:?}", error);
                        }
                    });
                },
            ));
            loop {
                scheduler.tick();
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        });
        Ok(())
    }
}
