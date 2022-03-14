CREATE TABLE IF NOT EXISTS sub_event_heartbeat_received
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         integer,
    event_index             integer NOT NULL,
    session_index           bigint NOT NULL,
    im_online_key           VARCHAR(66) NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_event_heartbeat_received
    ADD CONSTRAINT sub_event_heartbeat_received_fk_block
    FOREIGN KEY (block_hash)
        REFERENCES sub_block (hash)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_event_heartbeat_received
    ADD CONSTRAINT sub_event_heartbeat_received_fk_validator_account_id
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;

CREATE INDEX sub_event_heartbeat_received_idx_block_hash
    ON sub_event_heartbeat_received (block_hash);
CREATE INDEX sub_event_heartbeat_received_idx_validator_account_id
    ON sub_event_heartbeat_received (validator_account_id);
CREATE INDEX sub_event_heartbeat_received_idx_session_index
    ON sub_event_heartbeat_received (session_index);
CREATE INDEX sub_event_heartbeat_received_idx_session_index_account_id
    ON sub_event_heartbeat_received (validator_account_id, session_index);