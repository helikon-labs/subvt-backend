use crate::postgres::PostgreSQLStorage;
use std::str::FromStr;
use subvt_types::report::{EraReport, EraValidatorReport};
use subvt_types::substrate::Era;

type PostgresEraValidatorReport = (
    Option<i64>,
    Option<i64>,
    Option<bool>,
    Option<i64>,
    Option<String>,
    Option<String>,
    i32,
    Option<i32>,
    i64,
    i64,
    i32,
    i64,
    i32,
);

type PostgresEraReport = (
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<i64>,
    i64,
    Option<String>,
    Option<i32>,
    i32,
    i64,
    i32,
);

fn parse_maybe_string<T: FromStr>(maybe_string: &Option<String>) -> Result<Option<T>, T::Err> {
    if let Some(string) = maybe_string {
        Ok(Some(string.parse::<T>()?))
    } else {
        Ok(None)
    }
}

impl PostgreSQLStorage {
    async fn get_single_era_validator_report(
        &self,
        era_index: u32,
        validator_account_id_hex_string: &str,
    ) -> anyhow::Result<Option<EraValidatorReport>> {
        let era_validator_report: PostgresEraValidatorReport = sqlx::query_as(
            r#"
            SELECT era_start_timestamp, era_end_timestamp, is_active, commission_per_billion, self_stake, total_stake, block_count, reward_points, self_reward, staker_reward, offline_offence_count, slashed_amount, chilling_count
            FROM sub_get_era_validator_report($1, $2)
            "#
        )
            .bind(era_index as i64)
            .bind(validator_account_id_hex_string)
            .fetch_one(&self.connection_pool)
            .await?;
        let maybe_era = if era_validator_report.0.is_some() & era_validator_report.1.is_some() {
            Some(Era {
                index: era_index,
                start_timestamp: era_validator_report.0.unwrap() as u64,
                end_timestamp: era_validator_report.1.unwrap() as u64,
            })
        } else {
            None
        };
        if let Some(era) = maybe_era {
            Ok(Some(EraValidatorReport {
                era,
                is_active: era_validator_report.2,
                commission_per_billion: era_validator_report.3.map(|value| value as u32),
                self_stake: parse_maybe_string(&era_validator_report.4)?,
                total_stake: parse_maybe_string(&era_validator_report.5)?,
                block_count: era_validator_report.6 as u32,
                reward_points: era_validator_report.7.map(|value| value as u128),
                self_reward: era_validator_report.8 as u128,
                staker_reward: era_validator_report.9 as u128,
                offline_offence_count: era_validator_report.10 as u16,
                slashed_amount: era_validator_report.11 as u128,
                chilling_count: era_validator_report.12 as u16,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_era_validator_report(
        &self,
        start_era_index: u32,
        end_era_index: u32,
        validator_account_id_hex_string: &str,
    ) -> anyhow::Result<Vec<EraValidatorReport>> {
        if start_era_index > end_era_index {
            return Ok(Vec::new());
        }
        let era_reports = {
            let mut era_reports = Vec::new();
            for era_index in start_era_index..=end_era_index {
                if let Some(report) = self
                    .get_single_era_validator_report(era_index, validator_account_id_hex_string)
                    .await?
                {
                    era_reports.push(report)
                }
            }
            era_reports
        };
        Ok(era_reports)
    }

    async fn get_single_era_report(&self, era_index: u32) -> anyhow::Result<Option<EraReport>> {
        let era_report: PostgresEraReport = sqlx::query_as(
            r#"
            SELECT start_timestamp, end_timestamp, minimum_stake, maximum_stake, average_stake, median_stake, total_validator_reward, total_reward_points, total_reward, total_stake, active_nominator_count, offline_offence_count, slashed_amount, chilling_count
            FROM sub_get_era_report($1)
            "#
        )
            .bind(era_index as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        let maybe_era = if era_report.0.is_some() & era_report.1.is_some() {
            Some(Era {
                index: era_index,
                start_timestamp: era_report.0.unwrap() as u64,
                end_timestamp: era_report.1.unwrap() as u64,
            })
        } else {
            None
        };
        if let Some(era) = maybe_era {
            Ok(Some(EraReport {
                era,
                minimum_stake: parse_maybe_string(&era_report.2)?,
                maximum_stake: parse_maybe_string(&era_report.3)?,
                average_stake: parse_maybe_string(&era_report.4)?,
                median_stake: parse_maybe_string(&era_report.5)?,
                total_validator_reward: parse_maybe_string(&era_report.6)?,
                total_reward_points: era_report.7.map(|value| value as u128),
                total_reward: era_report.8 as u128,
                total_stake: parse_maybe_string(&era_report.9)?,
                active_nominator_count: era_report.10.map(|value| value as u64),
                offline_offence_count: era_report.11 as u64,
                slashed_amount: era_report.12 as u128,
                chilling_count: era_report.13 as u64,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_era_report(
        &self,
        start_era_index: u32,
        end_era_index: u32,
    ) -> anyhow::Result<Vec<EraReport>> {
        if start_era_index > end_era_index {
            return Ok(Vec::new());
        }
        let era_reports = {
            let mut era_reports = Vec::new();
            for era_index in start_era_index..=end_era_index {
                if let Some(report) = self.get_single_era_report(era_index).await? {
                    era_reports.push(report)
                }
            }
            era_reports
        };
        Ok(era_reports)
    }
}
