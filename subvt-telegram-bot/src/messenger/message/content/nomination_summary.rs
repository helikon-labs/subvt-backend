use super::MessageType;
use crate::CONFIG;
use subvt_types::subvt::{NominationSummary, ValidatorDetails};
use subvt_utility::numeric::format_decimal;
use tera::Context;

impl MessageType {
    pub(crate) fn fill_nomination_summary_context(
        &self,
        context: &mut Context,
        validator_details: &ValidatorDetails,
    ) {
        let self_stake = validator_details.self_stake.total_amount;
        let nomination_summary: NominationSummary = validator_details.into();
        let self_stake_formatted = format_decimal(
            self_stake,
            CONFIG.substrate.token_decimals,
            CONFIG.substrate.token_format_decimal_points,
        );
        let active_nomination_formatted = format_decimal(
            nomination_summary.active_nomination_total,
            CONFIG.substrate.token_decimals,
            CONFIG.substrate.token_format_decimal_points,
        );
        let inactive_nomination_formatted = format_decimal(
            nomination_summary.inactive_nomination_total,
            CONFIG.substrate.token_decimals,
            CONFIG.substrate.token_format_decimal_points,
        );
        let validator_display = validator_details
            .account
            .get_display_or_condensed_address(None);
        context.insert("validator_display", &validator_display);
        context.insert("token_ticker", &CONFIG.substrate.token_ticker);
        context.insert("self_stake", &self_stake_formatted);
        context.insert("active_nomination_total", &active_nomination_formatted);
        context.insert(
            "active_nominator_count",
            &nomination_summary.active_nominator_count,
        );
        context.insert("inactive_nomination_total", &inactive_nomination_formatted);
        context.insert(
            "inactive_nominator_count",
            &nomination_summary.inactive_nominator_count,
        );
    }
}
