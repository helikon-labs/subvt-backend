//! Content for all message types.
use super::MessageType;
use crate::CONFIG;
use subvt_utility::text::get_condensed_address;
use tera::{Context, Tera};

mod network_status;
mod nomination_details;
mod nomination_summary;
mod referendum_details;
mod validator_info;
mod validators_summary;

impl MessageType {
    pub fn get_content(&self, renderer: &Tera) -> String {
        let mut context = Context::new();
        let template_name = match self {
            Self::About => {
                context.insert("chain", &CONFIG.substrate.chain);
                let maybe_version: Option<&str> = option_env!("CARGO_PKG_VERSION");
                if let Some(version) = maybe_version {
                    context.insert("version", version);
                }
                "about.html"
            }
            Self::Help => "help.html",
            Self::Intro => {
                context.insert("chain", &CONFIG.substrate.chain);
                "introduction.html"
            }
            Self::Ok => "ok.html",
            Self::BadRequest => "bad_request.html",
            Self::GenericError => "generic_error.html",
            Self::Broadcast => "broadcast.html",
            Self::BroadcastConfirm => "confirm.html",
            Self::UnknownCommand(command) => {
                context.insert("command", command);
                "unknown_command.html"
            }
            Self::InvalidAddress(address) => {
                context.insert("address", address);
                "invalid_address.html"
            }
            Self::InvalidAddressTryAgain(address) => {
                context.insert("address", address);
                "invalid_address_try_again.html"
            }
            Self::ValidatorNotFound { maybe_address } => {
                if let Some(address) = maybe_address {
                    context.insert("condensed_address", &get_condensed_address(address, None));
                }
                "validator_not_found.html"
            }
            Self::AddValidatorNotFound(address) => {
                context.insert("condensed_address", &get_condensed_address(address, None));
                "add_validator_not_found.html"
            }
            Self::ValidatorExistsOnChat(validator_display) => {
                context.insert("validator_display", validator_display);
                "validator_exists_on_chat.html"
            }
            Self::TooManyValidatorsOnChat => {
                context.insert(
                    "max_validators_per_chat",
                    &CONFIG.telegram_bot.max_validators_per_chat,
                );
                "too_many_validators_on_chat.html"
            }
            Self::NoValidatorsOnChat => "no_validators_on_chat.html",
            Self::ValidatorAdded => "validator_added.html",
            Self::AddValidator => "add_validator.html",
            Self::ValidatorList { .. } => "select_validator.html",
            Self::ValidatorInfo {
                address,
                maybe_validator_details,
                maybe_onekv_candidate_summary,
                missing_referendum_votes,
            } => {
                self.fill_validator_info_context(
                    &mut context,
                    address,
                    maybe_validator_details,
                    maybe_onekv_candidate_summary,
                    missing_referendum_votes,
                );
                "validator_info.html"
            }
            Self::NominationSummary {
                validator_details, ..
            } => {
                self.fill_nomination_summary_context(&mut context, validator_details);
                "nomination_summary.html"
            }
            Self::NominationDetails {
                validator_details,
                onekv_nominator_account_ids,
            } => {
                self.fill_nomination_details_context(
                    &mut context,
                    validator_details,
                    onekv_nominator_account_ids,
                );
                "nomination_details.html"
            }
            Self::AllValidatorsRemoved => "all_validators_removed.html",
            Self::ValidatorRemoved(validator) => {
                let display = if let Some(display) = &validator.display {
                    display.clone()
                } else {
                    get_condensed_address(&validator.address, None)
                };
                context.insert("display", &display);
                "validator_removed.html"
            }
            Self::Settings => "settings_prompt.html",
            Self::NetworkStatus(network_status) => {
                self.fill_network_status_context(&mut context, network_status);
                "network_status.html"
            }
            Self::NoPayoutsFound => "no_payouts_found.html",
            Self::NoRewardsFound => "no_rewards_found.html",
            Self::NoOpenReferendaFound(track) => {
                context.insert("track", track.name());
                context.insert("chain", &CONFIG.substrate.chain);
                "no_referenda_found.html"
            }
            Self::RemoveAllValidatorsConfirm => "confirm.html",
            Self::ReferendumList(_, _) => "select_referendum.html",
            Self::ReferendumTracks(_) => "select_referendum_track.html",
            Self::ReferendumNotFound(id) => {
                context.insert("referendum_id", &id);
                "referendum_not_found.html"
            }
            Self::ReferendumDetails {
                post,
                chat_validator_votes,
            } => {
                self.fill_referendum_details_context(&mut context, post, chat_validator_votes);
                "referendum_details.html"
            }
            Self::SelectContactType => "select_contact_type.html",
            Self::EnterBugReport => "enter_bug_report.html",
            Self::EnterFeatureRequest => "enter_feature_request.html",
            Self::ReportSaved => "report_saved.html",
            Self::BugReport(content) => {
                context.insert("content", &content);
                "bug_report.html"
            }
            Self::FeatureRequest(content) => {
                context.insert("content", &content);
                "feature_request.html"
            }
            Self::NFTs { total_count, .. } => {
                context.insert("total_count", total_count);
                "select_nft.html"
            }
            Self::NoNFTsForValidator => "no_nfts_for_validator.html",
            Self::Loading => "loading.html",
            Self::ValidatorsSummary(validator_summaries) => {
                self.fill_validators_summary_context(&mut context, validator_summaries);
                "validators_summary.html"
            }
        };
        renderer.render(template_name, &context).unwrap()
    }
}
