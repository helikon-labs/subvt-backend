-- telemetry node network stats
ALTER TABLE sub_telemetry_node_network_stats
    DROP CONSTRAINT sub_telemetry_node_network_stats_u_time_node_id;
-- extrinsic: nominate->validator
ALTER TABLE sub_extrinsic_nominate_validator
    DROP CONSTRAINT sub_extrinsic_nominate_validator_u_validator;
-- era staker
ALTER TABLE sub_era_staker
    DROP CONSTRAINT sub_era_staker_u_era_index_validator_nominator;
-- session para validator
ALTER TABLE sub_session_para_validator
    DROP CONSTRAINT sub_session_para_validator_u_session_index_validator;
-- era validator
ALTER TABLE sub_era_validator
    DROP CONSTRAINT sub_era_validator_u_era_index_validator_account_id;