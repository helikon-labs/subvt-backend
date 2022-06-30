<p align="center">
	<img width="400" src="https://raw.githubusercontent.com/helikon-labs/subvt/main/assets/design/logo/subvt_logo_blue.png">
</p>

# Testing the SubVT Telegram Bot

Please follow the steps to run the tests for the SubVT Telegram Bot.

1. Make sure you have [Docker](https://www.docker.com/) installed on your system.
2. Clone this repository `git clone https://github.com/helikon-labs/subvt-backend.git`.
3. Go to the bot's source [directory](.) `cd subvt-backend/subvt-telegram-bot`.
4. Make the test helper shell script executable by running `chmod +x test.sh`.
5. Run the test helper shell script [test.sh](./test.sh).
6. Helper script is going to:
   1. Start the necessary containers.
   2. Build the project for testing.
   3. Run the tests and display results.
   4. Remove the containers and resources.

# Test Cases

api::async_tests::async_send_message_success
api::async_tests::send_message_failure
test::basic::test_restore_chat_and_user
test::basic::test_save_new_chat
test::command::about::test_about
test::command::add_validator::test_add_non_existent_validator
test::command::add_validator::test_add_validator_duplicate
test::command::add_validator::test_add_validator_invalid_address
test::command::add_validator::test_add_validator_no_address
test::command::add_validator::test_add_validator_successful
test::command::broadcast::test_broadcast
test::command::broadcast::test_broadcast_and_broadcasttest_non_admin
test::command::broadcast::test_broadcasttest
test::command::cancel::test_start
test::command::contact::test_contact
test::command::democracy::test_democracy
test::command::help::test_help
test::command::invalid::test_invalid_command
test::command::network_status::test_get_network_status_success
test::command::nfts::test_nfts_no_validator
test::command::nfts::test_nfts_single_validator_no_nfts
test::command::nfts::test_nfts_single_validator_with_nfts
test::command::nomination_details::test_nomination_details_multiple_validators
test::command::nomination_details::test_nomination_details_no_validator
test::command::nomination_details::test_nomination_details_single_non_existent_validator
test::command::nomination_details::test_nomination_details_single_validator
test::command::nominations::test_nominations_multiple_validators
test::command::nominations::test_nominations_no_validator
test::command::nominations::test_nominations_single_non_existent_validator
test::command::nominations::test_nominations_single_validator
test::command::payouts::test_payouts_multiple_validators
test::command::payouts::test_payouts_no_validator
test::command::payouts::test_payouts_single_validator_no_payouts
test::command::remove::test_remove_multiple_validators
test::command::remove::test_remove_no_validator
test::command::remove::test_remove_single_validator
test::command::rewards::test_rewards_multiple_validators
test::command::rewards::test_rewards_no_validator
test::command::rewards::test_rewards_single_validator_no_rewards
test::command::settings::test_settings
test::command::start::test_start
test::command::validator_info::test_validator_info_multiple_validators
test::command::validator_info::test_validator_info_no_validator
test::command::validator_info::test_validator_info_single_validator
