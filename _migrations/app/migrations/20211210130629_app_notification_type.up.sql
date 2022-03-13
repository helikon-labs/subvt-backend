CREATE TABLE IF NOT EXISTS app_notification_type
(
    code        VARCHAR(256) PRIMARY KEY,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

INSERT INTO app_notification_type(code) VALUES('chain_validator_offline_offence');
INSERT INTO app_notification_type(code) VALUES('chain_validator_new_nomination');
INSERT INTO app_notification_type(code) VALUES('chain_validator_lost_nomination');
INSERT INTO app_notification_type(code) VALUES('chain_validator_nomination_amount_change');
INSERT INTO app_notification_type(code) VALUES('chain_validator_chilled');
INSERT INTO app_notification_type(code) VALUES('chain_validator_active');
INSERT INTO app_notification_type(code) VALUES('chain_validator_active_next_session');
INSERT INTO app_notification_type(code) VALUES('chain_validator_inactive');
INSERT INTO app_notification_type(code) VALUES('chain_validator_inactive_next_session');
INSERT INTO app_notification_type(code) VALUES('chain_validate_extrinsic');
INSERT INTO app_notification_type(code) VALUES('chain_validator_unclaimed_payout');
INSERT INTO app_notification_type(code) VALUES('chain_validator_block_authorship');
INSERT INTO app_notification_type(code) VALUES('chain_validator_set_controller');
INSERT INTO app_notification_type(code) VALUES('chain_validator_session_keys_changed');
INSERT INTO app_notification_type(code) VALUES('chain_validator_identity_changed');
INSERT INTO app_notification_type(code) VALUES('chain_validator_payout_stakers');
INSERT INTO app_notification_type(code) VALUES('telemetry_validator_offline');
INSERT INTO app_notification_type(code) VALUES('telemetry_validator_binary_out_of_date');
INSERT INTO app_notification_type(code) VALUES('telemetry_validator_peer_count_low');
INSERT INTO app_notification_type(code) VALUES('telemetry_validator_too_many_txs_in_queue');
INSERT INTO app_notification_type(code) VALUES('telemetry_validator_lagging');
INSERT INTO app_notification_type(code) VALUES('telemetry_validator_finality_lagging');
INSERT INTO app_notification_type(code) VALUES('telemetry_validator_download_bw_low');
INSERT INTO app_notification_type(code) VALUES('telemetry_validator_upload_bw_low');
INSERT INTO app_notification_type(code) VALUES('onekv_validator_binary_version_change');
INSERT INTO app_notification_type(code) VALUES('onekv_validator_rank_change');
INSERT INTO app_notification_type(code) VALUES('onekv_validator_location_change');
INSERT INTO app_notification_type(code) VALUES('onekv_validator_validity_change');

CREATE TYPE app_notification_type_param_data_type AS ENUM ('string', 'integer', 'balance', 'float', 'boolean');

CREATE TABLE IF NOT EXISTS app_notification_param_type
(
    id                      SERIAL PRIMARY KEY,
    notification_type_code  VARCHAR(256) NOT NULL,
    code                    VARCHAR(256) NOT NULL,
    "order"                 smallint NOT NULL,
    type                    app_notification_type_param_data_type NOT NULL,
    "min"                   VARCHAR(128),
    "max"                   VARCHAR(128),
    is_optional             boolean NOT NULL,
    description             text,
    CONSTRAINT app_notification_param_type_u_notification_type_order
            UNIQUE (notification_type_code, "order"),
    CONSTRAINT app_notification_param_type_u_notification_type_code
        UNIQUE (notification_type_code, code),
    CONSTRAINT app_notification_param_type_fk_notification_type
        FOREIGN KEY (notification_type_code)
            REFERENCES app_notification_type (code)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_notification_param_type_notification_type
        FOREIGN KEY (notification_type_code)
            REFERENCES app_notification_type (code)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- chain_validator_offline_offence :: no param
-- chain_validator_new_nomination
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'chain_validator_new_nomination',
    'minimum_amount',
    0,
    'balance',
    '0',
    NULL,
    true,
    'Minimum nomination amount in native token.'
);
-- chain_lost_nomination
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'chain_validator_lost_nomination',
    'minimum_amount',
    0,
    'balance',
    '0',
    NULL,
    true,
    'Minimum nomination amount in native token.'
);
-- chain_validator_nomination_amount_change :: no param
-- chain_validator_chilled :: no param
-- chain_validator_active :: no param
-- chain_validator_active_next_session :: no param
-- chain_validator_inactive :: no param
-- chain_validator_inactive_next_session :: no param
-- chain_validate_extrinsic :: no param
-- chain_validator_unclaimed_payout :: no param
-- chain_validator_block_authorship :: no param
-- chain_validator_set_controller :: no param
-- chain_validator_session_keys_changed :: no param
-- chain_validator_identity_changed :: no param
-- chain_validator_reward_payout :: no param
-- telemetry_validator_offline
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_offline',
    'duration_sec',
    0,
    'integer',
    '60',
    NULL,
    false,
    'Telemetry offline duration for the validator in seconds.'
);
-- telemetry_validator_binary_out_of_date
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_binary_out_of_date',
    'duration_sec',
    0,
    'integer',
    '60',
    NULL,
    false,
    'Telemetry binary out-of-date duration for the validator in seconds.'
);
-- telemetry_validator_peer_count_low
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_peer_count_low',
    'peer_count',
    0,
    'integer',
    '1',
    NULL,
    false,
    'Notification happens if the peer count is below this value for the given duration.'
);
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_peer_count_low',
    'duration_sec',
    1,
    'integer',
    '10',
    NULL,
    false,
    'Notification happens if the peer count is below the given number for the duration of this many seconds.'
);
-- telemetry_validator_too_many_txs_in_queue
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_too_many_txs_in_queue',
    'tx_count',
    0,
    'integer',
    '5',
    NULL,
    false,
    'Notification happens if the queued transaction count is above this value for the given duration.'
);
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_too_many_txs_in_queue',
    'duration_sec',
    1,
    'integer',
    '10',
    NULL,
    false,
    'Notification happens if the transaction count is above the given number for the duration of this many seconds.'
);
-- telemetry_validator_lagging
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_lagging',
    'block_count',
    0,
    'integer',
    '5',
    NULL,
    false,
    'Notification happens if the validator is behind this many blocks for the given duration.'
);
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_lagging',
    'duration_sec',
    1,
    'integer',
    '10',
    NULL,
    false,
    'Notification happens if the validator is behind the given number of blocks for the duration of this many seconds.'
);
-- telemetry_validator_finality_lagging
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_finality_lagging',
    'block_count',
    0,
    'integer',
    '5',
    NULL,
    false,
    'Notification happens if the validator''s finality is behind this many blocks for the given duration.'
);
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_finality_lagging',
    'duration_sec',
    1,
    'integer',
    '10',
    NULL,
    false,
    'Notification happens if the validator''s finality is behind the given number of blocks for the duration of this many seconds.'
);
-- telemetry_validator_download_bw_low
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_download_bw_low',
    'kilo_bits_per_second',
    0,
    'integer',
    '10240', -- 10 kilobytes per second
    NULL,
    false,
    'Notification happens if the validator''s download bandwidth is lower than this value in kilobits per second for the given duration.'
);
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_download_bw_low',
    'duration_sec',
    1,
    'integer',
    '10',
    NULL,
    false,
    'Notification happens if the validator''s download bandwidth is lower than this value in kBps for this many seconds.'
);
-- telemetry_validator_upload_bw_low
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_upload_bw_low',
    'kilo_bits_per_second',
    0,
    'integer',
    '10240', -- 10 kilobytes per second
    NULL,
    false,
    'Notification happens if the validator''s upload bandwidth is lower than this value in kilobits per second for the given duration.'
);
INSERT INTO app_notification_param_type(
    notification_type_code,
    code,
    "order",
    type,
    "min",
    "max",
    is_optional,
    description
) VALUES(
    'telemetry_validator_upload_bw_low',
    'duration_sec',
    1,
    'integer',
    '10',
    NULL,
    false,
    'Notification happens if the validator''s upload bandwidth is lower than this value in kilobits per second for this many seconds.'
);
-- onekv_validator_binary_version_change :: no param
-- onekv_validator_rank_change :: no param
-- onekv_validator_location_change :: no param
-- onekv_validator_validity_change :: no param