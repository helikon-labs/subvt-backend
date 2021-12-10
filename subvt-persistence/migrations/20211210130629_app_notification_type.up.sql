CREATE TABLE IF NOT EXISTS app_notification_type
(
    id          SERIAL PRIMARY KEY,
    code        VARCHAR(256) NOT NULL,
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