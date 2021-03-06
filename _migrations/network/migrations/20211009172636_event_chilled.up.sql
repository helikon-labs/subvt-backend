CREATE TABLE IF NOT EXISTS sub_event_chilled
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         INTEGER,
    nesting_index           text,
    event_index             INTEGER NOT NULL,
    stash_account_id        VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_chilled_u_event
        UNIQUE (block_hash, event_index, stash_account_id),
    CONSTRAINT sub_event_chilled_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_chilled_fk_account
        FOREIGN KEY (stash_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_chilled_idx_block_hash
    ON sub_event_chilled (block_hash);