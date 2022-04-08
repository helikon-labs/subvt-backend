use super::MessageType;
use crate::CONFIG;
use chrono::{TimeZone, Utc};
use subvt_types::onekv::OneKVCandidateSummary;
use subvt_types::subvt::ValidatorDetails;
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::{get_condensed_address, get_condensed_session_keys};
use tera::Context;

impl MessageType {
    pub(crate) fn fill_validator_info_context(
        &self,
        context: &mut Context,
        address: &str,
        maybe_validator_details: &Option<ValidatorDetails>,
        maybe_onekv_candidate_summary: &Option<OneKVCandidateSummary>,
    ) {
        context.insert("condensed_address", &get_condensed_address(address, None));
        context.insert("is_validator", &maybe_validator_details.is_some());
        if let Some(validator_details) = maybe_validator_details {
            if let Some(display) = validator_details.account.get_full_display() {
                context.insert("has_display", &true);
                context.insert("display", &display);
            } else {
                context.insert("has_display", &false);
            }
            context.insert("network", &CONFIG.substrate.chain);
            let address = &validator_details.account.address;
            context.insert("address", address);
            context.insert("condensed_address", &get_condensed_address(address, None));
            let controller_address = validator_details.controller_account_id.to_ss58_check();
            context.insert("controller_address", &controller_address);
            context.insert(
                "condensed_controller_address",
                &get_condensed_address(&controller_address, None),
            );
            context.insert(
                "condensed_session_keys",
                &get_condensed_session_keys(&validator_details.next_session_keys).to_lowercase(),
            );
            context.insert("is_active", &validator_details.is_active);
            context.insert("is_para_validator", &validator_details.is_para_validator);
            context.insert(
                "is_active_next_session",
                &validator_details.active_next_session,
            );
            context.insert(
                "commission",
                &format_decimal(
                    validator_details.preferences.commission_per_billion as u128,
                    7,
                    2,
                ),
            );
            context.insert(
                "blocks_nominations",
                &validator_details.preferences.blocks_nominations,
            );
            context.insert("oversubscribed", &validator_details.oversubscribed);
            if let Some(heartbeat_received) = validator_details.heartbeat_received {
                context.insert("heartbeat_received", &heartbeat_received);
            }
            context.insert("slash_count", &validator_details.slash_count);
        }
        context.insert("is_onekv", &maybe_onekv_candidate_summary.is_some());
        if let Some(onekv_summary) = maybe_onekv_candidate_summary {
            context.insert("onekv_name", &onekv_summary.name);
            if let Some(location) = &onekv_summary.location {
                context.insert("onekv_location", location);
            }
            let date_time_format = "%b %d, %Y %H:%M UTC";
            let discovered_at = Utc::timestamp(&Utc, onekv_summary.discovered_at as i64 / 1000, 0);
            context.insert(
                "onekv_discovered_at",
                &discovered_at.format(date_time_format).to_string(),
            );
            if let Some(version) = &onekv_summary.version {
                context.insert("onekv_version", version);
            }
            if let Some(nominated_at) = onekv_summary.nominated_at {
                let nominated_at = Utc::timestamp(&Utc, nominated_at as i64 / 1000, 0);
                context.insert(
                    "onekv_nominated_at",
                    &nominated_at.format(date_time_format).to_string(),
                );
            }
            if onekv_summary.online_since > 0 {
                let online_since =
                    Utc::timestamp(&Utc, onekv_summary.online_since as i64 / 1000, 0);
                context.insert(
                    "onekv_online_since",
                    &online_since.format(date_time_format).to_string(),
                );
            } else if onekv_summary.offline_since > 0 {
                let offline_since =
                    Utc::timestamp(&Utc, onekv_summary.offline_since as i64 / 1000, 0);
                context.insert(
                    "onekv_offline_since",
                    &offline_since.format(date_time_format).to_string(),
                );
            }
            if let Some(rank) = onekv_summary.rank {
                context.insert("onekv_rank", &rank);
            }
            if let Some(score) = onekv_summary.total_score {
                context.insert("onekv_score", &(score as u64));
            }
            let is_valid = onekv_summary.is_valid();
            context.insert("onekv_is_valid", &is_valid);
            if !is_valid {
                let invalidity_reasons: Vec<String> = onekv_summary
                    .validity
                    .iter()
                    .filter(|v| !v.is_valid)
                    .map(|v| v.details.clone())
                    .collect();
                context.insert("onekv_invalidity_reasons", &invalidity_reasons);
            }
            context.insert(
                "onekv_democracy_vote_count",
                &onekv_summary.democracy_vote_count,
            );
            context.insert(
                "onekv_council_vote_count",
                &onekv_summary.council_votes.len(),
            );
            let last_updated =
                Utc::timestamp(&Utc, onekv_summary.record_created_at as i64 / 1000, 0);
            context.insert(
                "onekv_last_updated",
                &last_updated.format(date_time_format).to_string(),
            );
        }
    }
}
