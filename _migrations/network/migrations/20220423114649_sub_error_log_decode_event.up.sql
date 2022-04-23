CREATE TABLE IF NOT EXISTS sub_error_log_process_event
(
    id              SERIAL PRIMARY KEY,
    block_hash      VARCHAR(66) NOT NULL,
    block_number    bigint NOT NULL,
    event_index     integer NOT NULL,
    type            VARCHAR(32) NOT NULL,
    error_log       text,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_error_log_process_extrinsic_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_error_log_process_event_idx_block_hash
    ON sub_error_log_process_event (block_hash);
