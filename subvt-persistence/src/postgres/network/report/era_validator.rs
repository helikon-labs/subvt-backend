use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::report::EraValidatorReport;
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

impl PostgreSQLNetworkStorage {
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
                self_stake: super::parse_maybe_string(&era_validator_report.4)?,
                total_stake: super::parse_maybe_string(&era_validator_report.5)?,
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
}
