CREATE TABLE IF NOT EXISTS sub_extrinsic_heartbeat
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         integer NOT NULL,
    is_nested_call          boolean NOT NULL,
    block_number            bigint NOT NULL,
    session_index           bigint NOT NULL,
    validator_index         bigint NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    is_successful           boolean NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_extrinsic_heartbeat_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_extrinsic_heartbeat_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

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