CREATE TABLE IF NOT EXISTS sub_event_killed_account
(
    id              SERIAL PRIMARY KEY,
    block_hash      VARCHAR(66)                 NOT NULL,
    extrinsic_index integer,
    event_index     integer NOT NULL,
    account_id      VARCHAR(66)                 NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT extrinsic_killed_account_u_block_hash_account_id
        UNIQUE (block_hash, account_id),
    CONSTRAINT extrinsic_killed_account_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT extrinsic_killed_account_fk_account
        FOREIGN KEY (account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX sub_event_killed_account_idx_block_hash
    ON sub_event_killed_account (block_hash);