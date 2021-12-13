CREATE TABLE IF NOT EXISTS app_notification_type
(
    code        VARCHAR(256) PRIMARY KEY,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

INSERT INTO app_notification_type(code) VALUES('chain_offline_offence');
INSERT INTO app_notification_type(code) VALUES('chain_new_nomination');
INSERT INTO app_notification_type(code) VALUES('chain_nomination_lost');
INSERT INTO app_notification_type(code) VALUES('chain_chilling');
INSERT INTO app_notification_type(code) VALUES('chain_active_set_inclusion');
INSERT INTO app_notification_type(code) VALUES('chain_active_set_exclusion');
INSERT INTO app_notification_type(code) VALUES('chain_commission_change');
INSERT INTO app_notification_type(code) VALUES('chain_unclaimed_payout');
INSERT INTO app_notification_type(code) VALUES('chain_block_authorship');
INSERT INTO app_notification_type(code) VALUES('telemetry_node_offline');
INSERT INTO app_notification_type(code) VALUES('telemetry_node_binary_out_of_date');
INSERT INTO app_notification_type(code) VALUES('telemetry_node_peer_count_low');
INSERT INTO app_notification_type(code) VALUES('telemetry_node_too_many_txs_in_queue');
INSERT INTO app_notification_type(code) VALUES('telemetry_node_lagging');
INSERT INTO app_notification_type(code) VALUES('telemetry_node_finality_lagging');
INSERT INTO app_notification_type(code) VALUES('telemetry_node_download_bw_low');
INSERT INTO app_notification_type(code) VALUES('telemetry_node_upload_bw_low');
INSERT INTO app_notification_type(code) VALUES('onekv_rank_change');
INSERT INTO app_notification_type(code) VALUES('onekv_validity_change');

CREATE TYPE app_notification_type_param_data_type AS ENUM ('string', 'integer', 'balance', 'float', 'boolean');

CREATE TABLE IF NOT EXISTS app_notification_type_param
(
    notification_type_code VARCHAR(256),
    code VARCHAR(256),
    type app_notification_type_param_data_type NOT NULL,
    "min" VARCHAR(128),
    "max" VARCHAR(128),
    is_optional boolean NOT NULL,
    --CONSTRAINT app_notification_type_param_u_public_key UNIQUE (public_key_hex),
    CONSTRAINT app_notification_type_param_pk
        PRIMARY KEY (notification_type_code, code),
    CONSTRAINT app_notification_type_param_fk_notification_type
        FOREIGN KEY (notification_type_code)
            REFERENCES app_notification_type (code)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_notification_type_param_notification_type
        FOREIGN KEY (notification_type_code)
            REFERENCES app_notification_type (code)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- new nomination
INSERT INTO app_notification_type_param(
    notification_type_code,
    code,
    type,
    "min",
    "max",
    is_optional
) VALUES(
    'chain_new_nomination',
    'min_amount',
    'balance',
    NULL,
    NULL,
    true
);

-- chain_nomination_lost
-- chain_chilling
-- chain_active_set_inclusion
-- chain_active_set_exclusion
-- chain_commission_change
-- chain_unclaimed_payout
-- chain_block_authorship
-- telemetry_node_offline
-- telemetry_node_binary_out_of_date
-- telemetry_node_peer_count_low
-- telemetry_node_too_many_txs_in_queue
-- telemetry_node_lagging
-- telemetry_node_finality_lagging
-- telemetry_node_download_bw_low
-- telemetry_node_upload_bw_low
-- onekv_rank_change
-- onekv_validity_change