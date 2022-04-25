use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::report::EraReport;
use subvt_types::substrate::Era;

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

impl PostgreSQLNetworkStorage {
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
                minimum_stake: super::parse_maybe_string(&era_report.2)?,
                maximum_stake: super::parse_maybe_string(&era_report.3)?,
                average_stake: super::parse_maybe_string(&era_report.4)?,
                median_stake: super::parse_maybe_string(&era_report.5)?,
                total_validator_reward: super::parse_maybe_string(&era_report.6)?,
                total_reward_points: era_report.7.map(|value| value as u128),
                total_reward: era_report.8 as u128,
                total_stake: super::parse_maybe_string(&era_report.9)?,
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
