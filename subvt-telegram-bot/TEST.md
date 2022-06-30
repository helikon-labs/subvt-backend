<p align="center">
	<img width="400" src="https://raw.githubusercontent.com/helikon-labs/subvt/main/assets/design/logo/subvt_logo_blue.png">
</p>

# Testing the SubVT Telegram Bot

Please follow the steps to run the tests for the SubVT Telegram Bot.

1. Make sure you have [Docker](https://www.docker.com/) installed on your system.
2. Clone this repository `git clone https://github.com/helikon-labs/subvt-backend.git`.
3. Go to the bot's source [directory](.) `cd subvt-backend/subvt-telegram-bot`.
4. Make the test helper shell script executable by running `chmod +x test.sh`.
5. Run the test helper shell script [test.sh](./test.sh). You may need to run it with `sudo` privileges on Linux systems.
6. Helper script is going to:
   1. Start the necessary containers.
   2. Build the project for testing.
   3. Run the tests and display results.
   4. Remove the containers and resources.

## Test Cases

Please see the test function commants in the files listed below for test case descriptions.

| File                                                                                 | Test Case                                               |
|--------------------------------------------------------------------------------------|---------------------------------------------------------|
| [./src/api/mod.rs](./src/api/mod.rs)                                                 | `test_async_send_message_success`                       |
|                                                                                      | `test_send_message_failure`                             |
| [./src/test/basic/mod.rs](./src/test/basic/mod.rs)                                   | `test_restore_chat_and_user`                            |
|                                                                                      | `test_save_new_chat`                                    |
| [./src/test/command/about.rs](./src/test/command/about.rs)                           | `test_about`                                            |
| [./src/test/command/add_validator.rs](./src/test/command/add_validator.rs)           | `test_add_non_existent_validator`                       |
|                                                                                      | `test_add_validator_duplicate`                          |
|                                                                                      | `test_add_validator_invalid_address`                    |
|                                                                                      | `test_add_validator_no_address`                         |
|                                                                                      | `test_add_validator_successful`                         |
| [./src/test/command/broadcast.rs](./src/test/command/broadcast.rs)                   | `test_broadcast`                                        |
|                                                                                      | `test_broadcast_and_broadcasttest_non_admin`            |
|                                                                                      | `test_broadcasttest`                                    |
| [./src/test/command/cancel.rs](./src/test/command/cancel.rs)                         | `test_cancel`                                           |
| [./src/test/command/contact.rs](./src/test/command/contact.rs)                       | `test_contact`                                          |
| [./src/test/command/democracy.rs](./src/test/command/democracy.rs)                   | `test_democracy`                                        |
| [./src/test/command/help.rs](./src/test/command/help.rs)                             | `test_help`                                             |
| [./src/test/command/invalid.rs](./src/test/command/invalid.rs)                       | `test_invalid_command`                                  |
| [./src/test/command/network_status.rs](./src/test/command/network_status.rs)         | `test_get_network_status_success`                       |
| [./src/test/command/nfts.rs](./src/test/command/nfts.rs)                             | `test_nfts_no_validator`                                |
|                                                                                      | `test_nfts_single_validator_no_nfts`                    |
|                                                                                      | `test_nfts_single_validator_with_nfts`                  |
| [./src/test/command/nomination_details.rs](./src/test/command/nomination_details.rs) | `test_nomination_details_multiple_validators`           |
|                                                                                      | `test_nomination_details_no_validator`                  |
|                                                                                      | `test_nomination_details_single_non_existent_validator` |
|                                                                                      | `test_nomination_details_single_validator`              |
| [./src/test/command/nominations.rs](./src/test/command/nominations.rs)               | `test_nominations_multiple_validators`                  |
|                                                                                      | `test_nominations_no_validator`                         |
|                                                                                      | `test_nominations_single_non_existent_validator`        |
|                                                                                      | `test_nominations_single_validator`                     |
| [./src/test/command/payouts.rs](./src/test/command/payouts.rs)                       | `test_payouts_multiple_validators`                      |
|                                                                                      | `test_payouts_no_validator`                             |
|                                                                                      | `test_payouts_single_validator_no_payouts`              |
| [./src/test/command/remove.rs](./src/test/command/remove.rs)                         | `test_remove_multiple_validators`                       |
|                                                                                      | `test_remove_no_validator`                              |
|                                                                                      | `test_remove_single_validator`                          |
| [./src/test/command/rewards.rs](./src/test/command/rewards.rs)                       | `test_rewards_multiple_validators`                      |
|                                                                                      | `test_rewards_no_validator`                             |
|                                                                                      | `test_rewards_single_validator_no_rewards`              |
| [./src/test/command/settings.rs](./src/test/command/settings.rs)                     | `test_settings`                                         |
| [./src/test/command/start.rs](./src/test/command/start.rs)                           | `test_start`                                            |
| [./src/test/command/validator_info.rs](./src/test/command/validator_info.rs)         | `test_validator_info_multiple_validators`               |
|                                                                                      | `test_validator_info_no_validator`                      |
|                                                                                      | `test_validator_info_single_validator`                  |

## Room for Improvement

- Integration tests for reward and payout chart image generation.
- Template content generation tests for all message types with possible parameter combinations.
