-- app event onekv validity change
DROP INDEX sub_app_event_onekv_validity_change_idx_validator;
-- app event onekv rank change
DROP INDEX sub_app_event_onekv_rank_change_idx_validator;
-- app event validator inactive
DROP INDEX sub_app_event_validator_inactive_idx_validator;
-- app event validator active
DROP INDEX sub_app_event_validator_active_idx_validator;
-- app event validator inactive next session
DROP INDEX sub_app_event_validator_inactive_next_session_idx_validator;
-- app event validator active next session
DROP INDEX sub_app_event_validator_active_next_session_idx_validator;
-- app event nomination amount change
DROP INDEX sub_app_event_nomination_amount_change_idx_validator;
-- app event lost nomination
DROP INDEX sub_app_event_lost_nomination_idx_validator_account_id;
-- app event new nomination
DROP INDEX sub_app_event_new_nomination_idx_validator_account_id;
-- app event removed validator
DROP INDEX sub_app_event_removed_validator_idx_validator_account_id;
-- app event new validator
DROP INDEX sub_app_event_new_validator_idx_validator_account_id;
-- telemetry node
DROP INDEX sub_telemetry_node_idx_controller_account_id;
DROP INDEX sub_telemetry_node_idx_name;
-- onekv candidate fault evet
DROP INDEX sub_onekv_candidate_fault_event_idx_validator_account_id;
-- onekv candidate rank event
DROP INDEX sub_onekv_candidate_rank_event_idx_validator_account_id;
-- onekv candidate validity
DROP INDEX sub_onekv_candidate_validity_idx_one_kv_candidate_id;
DROP INDEX sub_onekv_candidate_validity_idx_validator_account_id;
DROP INDEX sub_onekv_candidate_validity_idx_one_kv_candidate_id_is_valid;
-- onekv candidate
DROP INDEX sub_onekv_candidate_idx_validator_account_id;
-- event: heartbeat received
DROP INDEX sub_event_heartbeat_received_idx_block_hash;
DROP INDEX sub_event_heartbeat_received_idx_validator_account_id;
DROP INDEX sub_event_heartbeat_received_idx_session_index;
DROP INDEX sub_event_heartbeat_received_idx_session_index_account_id;
-- event: batch completed
DROP INDEX sub_event_batch_completed_idx_block_hash;
-- event: batch interrupted
DROP INDEX sub_event_batch_interrupted_idx_block_hash;
-- event: batch item completed
DROP INDEX sub_event_batch_item_completed_idx_block_hash;
-- event: killed account
DROP INDEX sub_event_killed_account_idx_block_hash;
-- event: new account
DROP INDEX sub_event_new_account_idx_block_hash;
-- event: slashed
DROP INDEX sub_event_slashed_idx_block_hash;
DROP INDEX sub_event_slashed_idx_validator_account_id;
-- event: rewarded
DROP INDEX sub_event_rewarded_idx_block_hash;
DROP INDEX sub_event_rewarded_idx_rewardee_account_id;
DROP INDEX sub_event_rewarded_idx_extrinsic_index_block_hash_rewardee;
-- event: nominator kicked
DROP INDEX sub_event_nominator_kicked_idx_block_hash;
-- event: era paid
DROP INDEX sub_event_era_paid_idx_block_hash;
-- event: chilled
DROP INDEX sub_event_chilled_idx_block_hash;
-- event: validator offline
DROP INDEX sub_event_validator_offline_idx_block_hash;
DROP INDEX sub_event_validator_offline_idx_validator_account_id;
-- extrinsic: bond
DROP INDEX sub_extrinsic_bond_idx_block_hash;
DROP INDEX sub_extrinsic_bond_idx_stash_account_id;
DROP INDEX sub_extrinsic_bond_idx_controller_account_id;
DROP INDEX sub_extrinsic_bond_idx_caller_controller;
-- extrinsic: set controller
DROP INDEX sub_extrinsic_set_controller_idx_block_hash;
DROP INDEX sub_extrinsic_set_controller_idx_caller_account_id;
DROP INDEX sub_extrinsic_set_controller_idx_controller_account_id;
-- extrinsic: heartbeat
DROP INDEX sub_extrinsic_heartbeat_idx_block_hash;
DROP INDEX sub_extrinsic_heartbeat_idx_block_number;
DROP INDEX sub_extrinsic_heartbeat_idx_session_index;
DROP INDEX sub_extrinsic_heartbeat_idx_validator_account_id;
DROP INDEX sub_extrinsic_heartbeat_idx_validator_session_successful;
-- extrinsic: payout stakers
DROP INDEX sub_extrinsic_payout_stakers_idx_block_hash;
DROP INDEX sub_extrinsic_payout_stakers_idx_caller_account_id;
DROP INDEX sub_extrinsic_payout_stakers_idx_validator_account_id;
DROP INDEX sub_extrinsic_payout_stakers_idx_era_index;
DROP INDEX sub_extrinsic_payout_stakers_idx_is_successful;
DROP INDEX sub_extrinsic_payout_stakers_idx_validator_era_successful;
DROP INDEX sub_extrinsic_payout_stakers_idx_era_block_success;
-- extrinsic: validate
DROP INDEX sub_extrinsic_validate_idx_block_hash;
-- extrinsic: nominate->validator
DROP INDEX sub_extrinsic_nominate_validator_idx_validator_account_id;
-- extrinsic: nominate
DROP INDEX sub_extrinsic_nominate_idx_block_hash;
-- era staker
DROP INDEX sub_era_staker_idx_era_index;
DROP INDEX sub_era_staker_idx_validator_account_id;
DROP INDEX sub_era_staker_idx_nominator_account_id;
DROP INDEX sub_era_staker_idx_era_index_validator_account_id;
-- block
DROP INDEX sub_block_idx_epoch_index;
DROP INDEX sub_block_idx_era_index;
DROP INDEX sub_block_idx_number;
DROP INDEX sub_block_idx_hash_epoch_index;
DROP INDEX sub_block_idx_author_account_id;
DROP INDEX sub_block_idx_era_index_author_account_id;
-- session para validator
DROP INDEX sub_session_para_validator_idx_era_index;
DROP INDEX sub_session_para_validator_idx_session_index;
DROP INDEX sub_session_para_validator_idx_validator_account_id;
DROP INDEX sub_session_para_validator_idx_session_index_validator;
-- epoch
DROP INDEX sub_epoch_idx_era;
-- era validator
DROP INDEX sub_era_validator_idx_era_index;
DROP INDEX sub_era_validator_idx_validator_account_id;
DROP INDEX sub_era_validator_idx_era_index_validator_account_id;
DROP INDEX sub_era_validator_idx_is_active;
DROP INDEX sub_era_validator_idx_active_validator_index;
DROP INDEX sub_era_validator_idx_validator_account_id_is_active;
-- era
DROP INDEX sub_era_idx_time_interval;
-- account
DROP INDEX sub_account_idx_discovered_at_block_hash;
DROP INDEX sub_account_idx_id_discovered_at_block_hash;
DROP INDEX sub_account_idx_killed_at_block_hash;
DROP INDEX sub_account_idx_id_killed_at_block_hash;