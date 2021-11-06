CREATE TABLE IF NOT EXISTS extrinsic_heartbeat
(
    id                   SERIAL PRIMARY KEY,
    block_hash           VARCHAR(66)                 NOT NULL,
    extrinsic_index      integer                     NOT NULL,
    is_nested_call       boolean                     NOT NULL,
    block_number         bigint                      NOT NULL,
    session_index        bigint                      NOT NULL,
    validator_index      bigint                      NOT NULL,
    is_successful        boolean                     NOT NULL,
    last_updated         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT extrinsic_heartbeat_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX extrinsic_heartbeat_idx_block_number
    ON extrinsic_heartbeat (block_number);

CREATE INDEX extrinsic_heartbeat_idx_session_index
    ON extrinsic_heartbeat (session_index);