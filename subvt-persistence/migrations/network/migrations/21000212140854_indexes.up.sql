-- account
CREATE INDEX sub_account_idx_discovered_at_block_hash
    ON sub_account (discovered_at_block_hash);
CREATE INDEX sub_account_idx_id_discovered_at_block_hash
    ON sub_account (id, discovered_at_block_hash);
CREATE INDEX sub_account_idx_killed_at_block_hash
    ON sub_account (killed_at_block_hash);
CREATE INDEX sub_account_idx_id_killed_at_block_hash
    ON sub_account (id, killed_at_block_hash);
-- era
CREATE INDEX sub_era_idx_time_interval
    ON sub_era (start_timestamp, end_timestamp);
-- era validator
CREATE INDEX sub_era_validator_idx_era_index
    ON sub_era_validator (era_index);
CREATE INDEX sub_era_validator_idx_validator_account_id
    ON sub_era_validator (validator_account_id);
CREATE INDEX sub_era_validator_idx_era_index_validator_account_id
    ON sub_era_validator (era_index, validator_account_id);
CREATE INDEX sub_era_validator_idx_is_active
    ON sub_era_validator (is_active);
CREATE INDEX sub_era_validator_idx_active_validator_index
    ON sub_era_validator (active_validator_index);
CREATE INDEX sub_era_validator_idx_validator_account_id_is_active
    ON sub_era_validator (validator_account_id, is_active);
-- epoch
CREATE INDEX sub_epoch_idx_era
    ON sub_epoch (era_index);
-- session para validator
CREATE INDEX sub_session_para_validator_idx_era_index
    ON sub_session_para_validator (session_index);
CREATE INDEX sub_session_para_validator_idx_session_index
    ON sub_session_para_validator (session_index);
CREATE INDEX sub_session_para_validator_idx_validator_account_id
    ON sub_session_para_validator (validator_account_id);
CREATE INDEX sub_session_para_validator_idx_session_index_validator_account_id
    ON sub_session_para_validator (session_index, validator_account_id);
-- block
CREATE INDEX sub_block_idx_epoch_index
    ON sub_block (epoch_index);
CREATE INDEX sub_block_idx_era_index
    ON sub_block (era_index);
CREATE INDEX sub_block_idx_number
    ON sub_block (number);
CREATE INDEX sub_block_idx_hash_epoch_index
    ON sub_block (hash, epoch_index);
CREATE INDEX sub_block_idx_author_account_id
    ON sub_block (author_account_id);
CREATE INDEX sub_block_idx_era_index_author_account_id
    ON sub_block (era_index, author_account_id);
-- era staker
CREATE INDEX sub_era_staker_idx_era_index
    ON sub_era_staker (era_index);
CREATE INDEX sub_era_staker_idx_validator_account_id
    ON sub_era_staker (validator_account_id);
CREATE INDEX sub_era_staker_idx_nominator_account_id
    ON sub_era_staker (nominator_account_id);
CREATE INDEX sub_era_staker_idx_era_index_validator_account_id
    ON sub_era_staker (era_index, validator_account_id);
-- extrinsic: nominate
CREATE INDEX sub_extrinsic_nominate_idx_block_hash
    ON sub_extrinsic_nominate (block_hash);
-- extrinsic: nominate->validator
CREATE INDEX sub_extrinsic_nominate_validator_idx_validator_account_id
    ON sub_extrinsic_nominate_validator (validator_account_id);
-- extrinsic: validate
CREATE INDEX sub_extrinsic_validate_idx_block_hash
    ON sub_extrinsic_validate (block_hash);
-- extrinsic: payout stakers
CREATE INDEX sub_extrinsic_payout_stakers_idx_block_hash
    ON sub_extrinsic_payout_stakers (block_hash);
CREATE INDEX sub_extrinsic_payout_stakers_idx_caller_account_id
    ON sub_extrinsic_payout_stakers (caller_account_id);
CREATE INDEX sub_extrinsic_payout_stakers_idx_validator_account_id
    ON sub_extrinsic_payout_stakers (validator_account_id);
CREATE INDEX sub_extrinsic_payout_stakers_idx_era_index
    ON sub_extrinsic_payout_stakers (era_index);
CREATE INDEX sub_extrinsic_payout_stakers_idx_is_successful
    ON sub_extrinsic_payout_stakers (is_successful);
CREATE INDEX sub_extrinsic_payout_stakers_idx_validator_era_successful
    ON sub_extrinsic_payout_stakers (validator_account_id, era_index, is_successful);
CREATE INDEX sub_extrinsic_payout_stakers_idx_index_era_index_block_hash_success
    ON sub_extrinsic_payout_stakers (era_index, extrinsic_index, block_hash, is_successful);
-- extrinsic: hearbeat
CREATE INDEX sub_extrinsic_heartbeat_idx_block_hash
    ON sub_extrinsic_heartbeat (block_hash);
CREATE INDEX sub_extrinsic_heartbeat_idx_block_number
    ON sub_extrinsic_heartbeat (block_number);
CREATE INDEX sub_extrinsic_heartbeat_idx_session_index
    ON sub_extrinsic_heartbeat (session_index);
CREATE INDEX sub_extrinsic_heartbeat_idx_validator_account_id
    ON sub_extrinsic_heartbeat (validator_account_id);
CREATE INDEX sub_extrinsic_heartbeat_idx_validator_account_id_session_index_is_successful
    ON sub_extrinsic_heartbeat (validator_account_id, session_index, is_successful);
-- extrinsic: set controller
CREATE INDEX sub_extrinsic_set_controller_idx_block_hash
    ON sub_extrinsic_set_controller (block_hash);
CREATE INDEX sub_extrinsic_set_controller_idx_caller_account_id
    ON sub_extrinsic_set_controller (caller_account_id);
CREATE INDEX sub_extrinsic_set_controller_idx_controller_account_id
    ON sub_extrinsic_set_controller (controller_account_id);
-- extinric: bond
CREATE INDEX sub_extrinsic_bond_idx_block_hash
    ON sub_extrinsic_bond (block_hash);
CREATE INDEX sub_extrinsic_bond_idx_stash_account_id
    ON sub_extrinsic_bond (stash_account_id);
CREATE INDEX sub_extrinsic_bond_idx_controller_account_id
    ON sub_extrinsic_bond (controller_account_id);
CREATE INDEX sub_extrinsic_bond_idx_caller_controller
    ON sub_extrinsic_bond (stash_account_id, controller_account_id);
-- event: validator offline
CREATE INDEX sub_event_validator_offline_idx_block_hash
    ON sub_event_validator_offline (block_hash);
CREATE INDEX sub_event_validator_offline_idx_validator_account_id
    ON sub_event_validator_offline (validator_account_id);
-- event: chilled
CREATE INDEX sub_event_chilled_idx_block_hash
    ON sub_event_chilled (block_hash);
-- event: era paid
CREATE INDEX sub_event_era_paid_idx_block_hash
    ON sub_event_era_paid (block_hash);
-- event: nominator kicked
CREATE INDEX sub_event_nominator_kicked_idx_block_hash
    ON sub_event_nominator_kicked (block_hash);
-- event: rewarded
CREATE INDEX sub_event_rewarded_idx_block_hash
    ON sub_event_rewarded (block_hash);
CREATE INDEX sub_event_rewarded_idx_rewardee_account_id
    ON sub_event_rewarded (rewardee_account_id);
CREATE INDEX sub_event_rewarded_idx_extrinsic_index_block_hash_rewardee
    ON sub_event_rewarded (extrinsic_index, block_hash, rewardee_account_id);
-- event: slashed
CREATE INDEX sub_event_slashed_idx_block_hash
    ON sub_event_slashed (block_hash);
CREATE INDEX sub_event_slashed_idx_validator_account_id
    ON sub_event_slashed (validator_account_id);
-- event: new account
CREATE INDEX sub_event_new_account_idx_block_hash
    ON sub_event_new_account (block_hash);
-- event: killed account
CREATE INDEX sub_event_killed_account_idx_block_hash
    ON sub_event_killed_account (block_hash);
-- event: batch item completed
CREATE INDEX sub_event_batch_item_completed_idx_block_hash
    ON sub_event_batch_item_completed (block_hash);
-- event: batch interrupted
CREATE INDEX sub_event_batch_interrupted_idx_block_hash
    ON sub_event_batch_interrupted (block_hash);
-- event: batch completed
CREATE INDEX sub_event_batch_completed_idx_block_hash
    ON sub_event_batch_completed (block_hash);
-- event: heartbeat received
CREATE INDEX sub_event_heartbeat_received_idx_block_hash
    ON sub_event_heartbeat_received (block_hash);
CREATE INDEX sub_event_heartbeat_received_idx_validator_account_id
    ON sub_event_heartbeat_received (validator_account_id);
CREATE INDEX sub_event_heartbeat_received_idx_session_index
    ON sub_event_heartbeat_received (session_index);
CREATE INDEX sub_event_heartbeat_received_idx_session_index_account_id
    ON sub_event_heartbeat_received (validator_account_id, session_index);
-- onekv candidate
CREATE INDEX sub_onekv_candidate_idx_validator_account_id
    ON sub_onekv_candidate (validator_account_id);
-- onekv candidate validity
CREATE INDEX sub_onekv_candidate_validity_idx_one_kv_candidate_id
    ON sub_onekv_candidate_validity (onekv_candidate_id);
CREATE INDEX sub_onekv_candidate_validity_idx_validator_account_id
    ON sub_onekv_candidate_validity (validator_account_id);
CREATE INDEX sub_onekv_candidate_validity_idx_one_kv_candidate_id_is_valid
    ON sub_onekv_candidate_validity (onekv_candidate_id, is_valid);
-- onekv candidate rank evet
CREATE INDEX sub_onekv_candidate_rank_event_idx_validator_account_id
    ON sub_onekv_candidate_rank_event (validator_account_id);
-- onekv candidate fault event
CREATE INDEX sub_onekv_candidate_fault_event_idx_validator_account_id
    ON sub_onekv_candidate_fault_event (validator_account_id);
-- telemetry node
CREATE INDEX sub_telemetry_node_idx_controller_account_id
    ON sub_telemetry_node (controller_account_id);
CREATE INDEX sub_telemetry_node_idx_name
    ON sub_telemetry_node (name);
-- app event new validator
CREATE INDEX sub_app_event_new_validator_idx_validator_account_id
    ON sub_app_event_new_validator (validator_account_id);
-- app event removed validator
CREATE INDEX sub_app_event_removed_validator_idx_validator_account_id
    ON sub_app_event_removed_validator (validator_account_id);
-- app event new nomination
CREATE INDEX sub_app_event_new_nomination_idx_validator_account_id
    ON sub_app_event_new_nomination (validator_account_id);
-- app event lost nomination
CREATE INDEX sub_app_event_lost_nomination_idx_validator_account_id
    ON sub_app_event_lost_nomination (validator_account_id);
-- app event nomination amount change
CREATE INDEX sub_app_event_nomination_amount_change_idx_validator
    ON sub_app_event_nomination_amount_change (validator_account_id);
-- app event validator active next session
CREATE INDEX sub_app_event_validator_active_next_session_idx_validator
    ON sub_app_event_validator_active_next_session (validator_account_id);
-- app event validator inactive next session
CREATE INDEX sub_app_event_validator_inactive_next_session_idx_validator
    ON sub_app_event_validator_inactive_next_session (validator_account_id);
-- app event validator active
CREATE INDEX sub_app_event_validator_active_idx_validator
    ON sub_app_event_validator_active (validator_account_id);
-- app event validator inactive
CREATE INDEX sub_app_event_validator_inactive_idx_validator
    ON sub_app_event_validator_inactive (validator_account_id);
-- app event onekv rank change
CREATE INDEX sub_app_event_onekv_rank_change_idx_validator
    ON sub_app_event_onekv_rank_change (validator_account_id);
-- app event onekv validity change
CREATE INDEX sub_app_event_onekv_validity_change_idx_validator
    ON sub_app_event_onekv_validity_change (validator_account_id);