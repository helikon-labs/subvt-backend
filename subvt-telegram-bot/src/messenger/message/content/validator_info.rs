//! Content for the `/validatorinfo` request, after the selection of the validator.
use super::MessageType;
use crate::CONFIG;
use subvt_types::dn::DNNode;
use subvt_types::subvt::ValidatorDetails;
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::{get_condensed_address, get_condensed_session_keys};
use tera::Context;

impl MessageType {
    pub(in crate::messenger::message) fn fill_validator_info_context(
        &self,
        context: &mut Context,
        address: &str,
        maybe_validator_details: &Option<ValidatorDetails>,
        maybe_dn_node: &Option<DNNode>,
        missing_referendum_votes: &Vec<u32>,
    ) {
        context.insert("chain", &CONFIG.substrate.chain);
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
                &validator_details.is_active_next_session,
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
        context.insert("missing_referendum_votes", missing_referendum_votes);
        context.insert("is_dn", &maybe_dn_node.is_some());
        if let Some(dn_node) = maybe_dn_node {
            context.insert("dn_identity", &dn_node.identity);
            context.insert("dn_status", &dn_node.status);
        }
    }
}
