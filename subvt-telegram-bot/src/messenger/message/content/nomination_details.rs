use super::MessageType;
use crate::CONFIG;
use itertools::Itertools;
use subvt_types::crypto::AccountId;
use subvt_types::subvt::ValidatorDetails;
use subvt_utility::numeric::format_decimal;
use tera::Context;

impl MessageType {
    pub(crate) fn fill_nomination_details_context(
        &self,
        context: &mut Context,
        validator_details: &ValidatorDetails,
        onekv_nominator_account_ids: &[AccountId],
    ) {
        let self_stake = validator_details.self_stake.total_amount;
        let self_stake_formatted = format_decimal(
            self_stake,
            CONFIG.substrate.token_decimals,
            CONFIG.substrate.token_format_decimal_points,
        );
        let validator_display = validator_details
            .account
            .get_display_or_condensed_address(Some(3));
        context.insert("validator_display", &validator_display);
        context.insert("token_ticker", &CONFIG.substrate.token_ticker);
        context.insert("self_stake", &self_stake_formatted);
        let mut active_nominator_account_ids = Vec::new();
        if let Some(active_stake) = &validator_details.validator_stake {
            let mut active_nomination_total = 0;
            let active_nominations: Vec<(String, String, bool)> = active_stake
                .nominators
                .iter()
                .map(|n| {
                    active_nomination_total += n.stake;
                    active_nominator_account_ids.push(n.account.id);
                    (
                        n.account.get_display_or_condensed_address(Some(3)),
                        n.stake,
                        onekv_nominator_account_ids.contains(&n.account.id),
                    )
                })
                .sorted_by(|n1, n2| n2.1.cmp(&n1.1))
                .map(|n| {
                    (
                        n.0,
                        format_decimal(
                            n.1,
                            CONFIG.substrate.token_decimals,
                            CONFIG.substrate.token_format_decimal_points,
                        ),
                        n.2,
                    )
                })
                .collect();
            let max_len = active_nominations.get(0).map(|n| n.1.len()).unwrap_or(0);
            context.insert(
                "active_nomination_total",
                &format_decimal(
                    active_nomination_total,
                    CONFIG.substrate.token_decimals,
                    CONFIG.substrate.token_format_decimal_points,
                ),
            );
            context.insert(
                "active_nominations",
                &active_nominations
                    .iter()
                    .map(|n| {
                        (
                            n.0.clone(),
                            format!("{}{}", " ".repeat(max_len - n.1.len()), n.1),
                            n.2,
                        )
                    })
                    .collect::<Vec<(String, String, bool)>>(),
            );
        }
        let mut inactive_nomination_total = 0;
        let inactive_nominations: Vec<(String, String, bool)> = validator_details
            .nominations
            .iter()
            .filter(|n| !active_nominator_account_ids.contains(&n.stash_account.id))
            .map(|n| {
                inactive_nomination_total += n.stake.active_amount;
                (
                    n.stash_account.get_display_or_condensed_address(Some(3)),
                    n.stake.active_amount,
                    onekv_nominator_account_ids.contains(&n.stash_account.id),
                )
            })
            .sorted_by(|n1, n2| n2.1.cmp(&n1.1))
            .map(|n| {
                (
                    n.0,
                    format_decimal(
                        n.1,
                        CONFIG.substrate.token_decimals,
                        CONFIG.substrate.token_format_decimal_points,
                    ),
                    n.2,
                )
            })
            .collect();
        if !inactive_nominations.is_empty() {
            let max_len = inactive_nominations.get(0).map(|n| n.1.len()).unwrap_or(0);
            context.insert(
                "inactive_nomination_total",
                &format_decimal(
                    inactive_nomination_total,
                    CONFIG.substrate.token_decimals,
                    CONFIG.substrate.token_format_decimal_points,
                ),
            );
            context.insert(
                "inactive_nominations",
                &inactive_nominations
                    .iter()
                    .map(|n| {
                        (
                            n.0.clone(),
                            format!("{}{}", " ".repeat(max_len - n.1.len()), n.1),
                            n.2,
                        )
                    })
                    .collect::<Vec<(String, String, bool)>>(),
            );
        }
    }
}
