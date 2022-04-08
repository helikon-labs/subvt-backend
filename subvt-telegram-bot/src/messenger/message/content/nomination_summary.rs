use super::MessageType;
use crate::CONFIG;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::Balance;
use subvt_types::subvt::ValidatorDetails;
use subvt_utility::numeric::format_decimal;
use tera::Context;

impl MessageType {
    pub(crate) fn fill_nomination_summary_context(
        &self,
        context: &mut Context,
        validator_details: &ValidatorDetails,
    ) {
        let self_stake = validator_details.self_stake.total_amount;
        let (
            active_nominator_count,
            active_nomination_total,
            inactive_nominator_count,
            inactive_nomination_total,
        ) = if let Some(validator_stake) = &validator_details.validator_stake {
            let active_nominator_account_ids: Vec<AccountId> = validator_stake
                .nominators
                .iter()
                .map(|n| n.account.id)
                .collect();
            let mut inactive_nominator_count: usize = 0;
            let mut inactive_nomination_total: Balance = 0;
            for nomination in &validator_details.nominations {
                if !active_nominator_account_ids.contains(&nomination.stash_account.id) {
                    inactive_nominator_count += 1;
                    inactive_nomination_total += nomination.stake.active_amount;
                }
            }
            (
                active_nominator_account_ids.len(),
                validator_stake.total_stake,
                inactive_nominator_count,
                inactive_nomination_total,
            )
        } else {
            let inactive_nomination_total: Balance = validator_details
                .nominations
                .iter()
                .map(|n| n.stake.total_amount)
                .sum();
            (
                0,
                0,
                validator_details.nominations.len(),
                inactive_nomination_total,
            )
        };

        let self_stake_formatted = format_decimal(
            self_stake,
            CONFIG.substrate.token_decimals,
            CONFIG.substrate.token_format_decimal_points,
        );
        let active_nomination_formatted = format_decimal(
            active_nomination_total,
            CONFIG.substrate.token_decimals,
            CONFIG.substrate.token_format_decimal_points,
        );
        let inactive_nomination_formatted = format_decimal(
            inactive_nomination_total,
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
        context.insert("active_nominator_count", &active_nominator_count);
        context.insert("inactive_nomination_total", &inactive_nomination_formatted);
        context.insert("inactive_nominator_count", &inactive_nominator_count);
    }
}
