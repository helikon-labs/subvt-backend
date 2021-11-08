CREATE TABLE IF NOT EXISTS event_heartbeat_received
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66)                 NOT NULL,
    session_index           bigint                      NOT NULL,
    extrinsic_index         integer,
    im_online_key           VARCHAR(66)                 NOT NULL,
    validator_account_id    VARCHAR(66)                 NOT NULL,
    last_updated            TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT event_heartbeat_received_u_block_hash_validator_account_id
        UNIQUE (block_hash, validator_account_id),
    CONSTRAINT event_heartbeat_received_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT event_heartbeat_received_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX event_heartbeat_received_idx_block_hash
    ON event_heartbeat_received (block_hash);

CREATE INDEX event_heartbeat_received_idx_validator_account_id
    ON event_heartbeat_received (validator_account_id);

CREATE INDEX event_heartbeat_received_idx_session_index
    ON event_heartbeat_received (session_index);

CREATE INDEX event_heartbeat_received_idx_session_index_account_id
    ON event_heartbeat_received (validator_account_id, session_index);