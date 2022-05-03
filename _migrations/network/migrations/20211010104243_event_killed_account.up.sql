CREATE TABLE IF NOT EXISTS sub_event_killed_account
(
    id              SERIAL PRIMARY KEY,
    block_hash      VARCHAR(66) NOT NULL,
    extrinsic_index integer,
    batch_index     text,
    event_index     integer NOT NULL,
    account_id      VARCHAR(66) NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_killed_account_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT event_killed_account_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT event_killed_account_fk_account
        FOREIGN KEY (account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_killed_account_idx_block_hash
    ON sub_event_killed_account (block_hash);