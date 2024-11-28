//! Updates the complete 1KV data for the network (only Polkadot and Kusama) on the database.
#![warn(clippy::disallowed_types)]

use async_trait::async_trait;
use lazy_static::lazy_static;
use std::cmp::max;
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;
use subvt_types::performance::SessionValidatorPerformance;
use subvt_types::report::ParaVotesSummary;
use subvt_types::substrate::Epoch;

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub struct SessionValidatorPerformanceUpdater;

impl SessionValidatorPerformanceUpdater {
    async fn process_session(
        &self,
        postgres: &PostgreSQLNetworkStorage,
        session: &Epoch,
    ) -> anyhow::Result<()> {
        let active_validator_account_ids = postgres
            .get_era_validator_account_ids(session.era_index, true)
            .await?;
        log::info!(
            "Process {} active validators in session {}.",
            active_validator_account_ids.len(),
            session.index
        );
        log::debug!("Get era validator records.");
        let mut era_active_validators = Vec::new();
        for account_id in active_validator_account_ids.iter() {
            let era_validator = match postgres
                .get_era_validator_by_session_index(account_id, session.index)
                .await?
            {
                Some(era_validator) => era_validator,
                _ => continue,
            };
            era_active_validators.push(era_validator);
        }
        log::debug!("Get performance data.");
        let mut session_validator_performances = Vec::new();
        for era_active_validator in era_active_validators.iter() {
            let mut performance = SessionValidatorPerformance {
                validator_account_id: era_active_validator.validator_account_id,
                era_index: session.era_index,
                session_index: session.index,
                active_validator_index: era_active_validator.active_validator_index.unwrap(),
                ..Default::default()
            };
            // block count
            performance.authored_block_count = postgres
                .get_number_of_blocks_in_epoch_by_validator(
                    session.index,
                    &era_active_validator.validator_account_id,
                )
                .await?;
            // para-related
            let maybe_para_validator = postgres
                .get_session_para_validator(
                    session.index,
                    &era_active_validator.validator_account_id,
                )
                .await?;
            if let Some(para_validator) = maybe_para_validator {
                performance.para_validator_group_index =
                    Some(para_validator.para_validator_group_index);
                performance.para_validator_index = Some(para_validator.para_validator_index);
                let votes = postgres
                    .get_session_para_validator_votes(
                        session.index,
                        para_validator.para_validator_index,
                    )
                    .await?;
                let votes_summary = ParaVotesSummary::from_para_votes(&votes);
                performance.implicit_attestation_count = Some(votes_summary.implicit);
                performance.explicit_attestation_count = Some(votes_summary.explicit);
                performance.missed_attestation_count = Some(votes_summary.missed);
                let attestation_count = (votes_summary.implicit + votes_summary.explicit) as u64;
                let total_attestation_slots =
                    (votes_summary.implicit + votes_summary.explicit + votes_summary.missed) as u64;
                let attestations_per_billion = if total_attestation_slots > 0 {
                    attestation_count * 1_000_000_000 / total_attestation_slots
                } else {
                    0
                };
                performance.attestations_per_billion = Some(attestations_per_billion as u32);
            }
            session_validator_performances.push(performance);
        }
        log::info!("Persist session performance data.");
        postgres
            .save_session_validator_performances(&session_validator_performances)
            .await?;
        log::info!(
            "Persisted {} validator performances for session {}.",
            session_validator_performances.len(),
            session.index
        );
        Ok(())
    }
}

#[async_trait(?Send)]
impl Service for SessionValidatorPerformanceUpdater {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.session_validator_performance_updater_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!(
            "Session validator performance updater has started with {} seconds refresh wait period.",
            CONFIG.session_validator_performance_updater.sleep_seconds,
        );
        let postgres =
            PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
        loop {
            let last_processed_session_index = postgres
                .get_session_validator_performance_updater_last_processed_session_id()
                .await?
                .unwrap_or(0);
            let current_session_index = postgres
                .get_current_epoch()
                .await?
                .map(|epoch| epoch.index)
                .unwrap_or(1);
            let start_session_index = max(
                last_processed_session_index + 1,
                CONFIG
                    .session_validator_performance_updater
                    .start_session_index,
            );
            log::info!(
                "Process sessions {}-{}.",
                start_session_index,
                current_session_index - 1,
            );
            if start_session_index >= (current_session_index - 1) {
                log::warn!(
                    "Start session index is greater than or equal to end session index. No-op."
                );
            } else {
                for session_index in start_session_index..current_session_index {
                    log::info!("Process session {session_index}.");
                    let session = match postgres.get_epoch_by_index(session_index).await? {
                        Some(session) => session,
                        _ => {
                            log::warn!("Session {session_index} not found in storage. Continue with the next session.");
                            continue;
                        }
                    };
                    self.process_session(&postgres, &session).await?;
                }
            }
            log::info!(
                "Sleep for {} seconds.",
                CONFIG.session_validator_performance_updater.sleep_seconds
            );
            tokio::time::sleep(std::time::Duration::from_secs(
                CONFIG.session_validator_performance_updater.sleep_seconds,
            ))
            .await;
        }
    }
}
