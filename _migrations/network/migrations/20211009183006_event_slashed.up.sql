CREATE TABLE IF NOT EXISTS sub_event_slashed
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         INTEGER,
    nesting_index           text,
    event_index             INTEGER NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    amount                  VARCHAR(128) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_slashed_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_slashed_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_slashed_fk_validator_account
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_slashed_idx_block_hash
    ON sub_event_slashed (block_hash);
CREATE INDEX IF NOT EXISTS sub_event_slashed_idx_validator_account_id
    ON sub_event_slashed (validator_account_id);