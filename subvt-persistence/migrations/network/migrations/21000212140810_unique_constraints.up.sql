-- era validator
ALTER TABLE sub_era_validator
    ADD CONSTRAINT sub_era_validator_u_era_index_validator_account_id
    UNIQUE (era_index, validator_account_id);
-- session para validator
ALTER TABLE sub_session_para_validator
    ADD CONSTRAINT sub_session_para_validator_u_session_index_validator
    UNIQUE (session_index, validator_account_id);
-- era staker
ALTER TABLE sub_era_staker
    ADD CONSTRAINT sub_era_staker_u_era_index_validator_nominator
    UNIQUE (era_index, validator_account_id, nominator_account_id);
-- extrinsic: nominate->validator
ALTER TABLE sub_extrinsic_nominate_validator
    ADD CONSTRAINT sub_extrinsic_nominate_validator_u_validator
    UNIQUE (extrinsic_nominate_id, validator_account_id);
-- telemetry node network stats
ALTER TABLE sub_telemetry_node_network_stats
    ADD CONSTRAINT sub_telemetry_node_network_stats_u_time_node_id
    UNIQUE (time, node_id);