CREATE TABLE IF NOT EXISTS event_killed_account
(
    id              SERIAL PRIMARY KEY,
    block_hash      VARCHAR(66)                 NOT NULL,
    extrinsic_index integer,
    account_id      VARCHAR(66)                 NOT NULL,
    last_updated    TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT extrinsic_killed_account_u_block_hash_account_id
        UNIQUE (block_hash, account_id),
    CONSTRAINT extrinsic_killed_account_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT extrinsic_killed_account_fk_account
        FOREIGN KEY (account_id)
            REFERENCES account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);
