//! Content for the `/networkstatus` request.
use super::MessageType;
use crate::CONFIG;
use chrono::Utc;
use subvt_types::subvt::NetworkStatus;
use subvt_utility::numeric::format_decimal;
use tera::Context;

impl MessageType {
    pub(in crate::messenger::message) fn fill_network_status_context(
        &self,
        context: &mut Context,
        network_status: &NetworkStatus,
    ) {
        let now = Utc::now();
        let date_time_format = "%b %d, %H:%M UTC";
        context.insert("chain", &CONFIG.substrate.chain);
        context.insert("token_ticker", &CONFIG.substrate.token_ticker);
        context.insert("best_block_hash", &network_status.best_block_hash);
        context.insert("best_block_number", &network_status.best_block_number);
        context.insert("finalized_block_hash", &network_status.finalized_block_hash);
        context.insert(
            "finalized_block_number",
            &network_status.finalized_block_number,
        );
        context.insert("active_era_index", &network_status.active_era.index);
        let era_start = network_status.active_era.get_start_date_time();
        // format!("{}{}", " ".repeat(max_len - n.1.len()), n.1),
        context.insert(
            "active_era_start",
            &era_start.format(date_time_format).to_string(),
        );
        let era_end = network_status.active_era.get_end_date_time();
        let duration_until_era_end = era_end - now;
        context.insert(
            "active_era_end",
            &era_end.format(date_time_format).to_string(),
        );
        context.insert(
            "hours_until_active_era_end",
            &(duration_until_era_end.num_seconds() / 60 / 60),
        );
        context.insert(
            "minutes_until_active_era_end",
            &((duration_until_era_end.num_seconds() / 60) % 60),
        );

        context.insert("current_epoch_index", &network_status.current_epoch.index);
        let epoch_start = network_status.current_epoch.get_start_date_time();
        context.insert(
            "current_epoch_start",
            &epoch_start.format(date_time_format).to_string(),
        );
        let epoch_end = network_status.current_epoch.get_end_date_time();
        let duration_until_epoch_end = epoch_end - now;
        context.insert(
            "current_epoch_end",
            &epoch_end.format(date_time_format).to_string(),
        );
        context.insert(
            "hours_until_current_epoch_end",
            &(duration_until_epoch_end.num_seconds() / 60 / 60),
        );
        context.insert(
            "minutes_until_current_epoch_end",
            &((duration_until_epoch_end.num_seconds() / 60) % 60),
        );

        context.insert(
            "active_validator_count",
            &network_status.active_validator_count,
        );
        context.insert(
            "inactive_validator_count",
            &network_status.inactive_validator_count,
        );
        context.insert(
            "return_rate_per_cent",
            &format_decimal(network_status.return_rate_per_million as u128, 4, 2),
        );
        // total stake
        let total_stake_formatted = format_decimal(
            network_status.total_stake,
            CONFIG.substrate.token_decimals,
            CONFIG.substrate.token_format_decimal_points,
        );
        context.insert("total_stake", &total_stake_formatted);
        // min stake
        let min_stake_formatted = {
            let min_stake_formatted = format_decimal(
                network_status.min_stake,
                CONFIG.substrate.token_decimals,
                CONFIG.substrate.token_format_decimal_points,
            );
            format!(
                "{}{}",
                " ".repeat(total_stake_formatted.len() - min_stake_formatted.len()),
                min_stake_formatted,
            )
        };
        context.insert("min_stake", &min_stake_formatted);
        // max stake
        let max_stake_formatted = {
            let max_stake_formatted = format_decimal(
                network_status.max_stake,
                CONFIG.substrate.token_decimals,
                CONFIG.substrate.token_format_decimal_points,
            );
            format!(
                "{}{}",
                " ".repeat(total_stake_formatted.len() - max_stake_formatted.len()),
                max_stake_formatted,
            )
        };
        context.insert("max_stake", &max_stake_formatted);
        // max stake
        let average_stake_formatted = {
            let average_stake_formatted = format_decimal(
                network_status.average_stake,
                CONFIG.substrate.token_decimals,
                CONFIG.substrate.token_format_decimal_points,
            );
            format!(
                "{}{}",
                " ".repeat(total_stake_formatted.len() - average_stake_formatted.len()),
                average_stake_formatted,
            )
        };
        context.insert("average_stake", &average_stake_formatted);
    }
}
