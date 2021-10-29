CREATE TABLE IF NOT EXISTS event_slashed
(
    id                   SERIAL PRIMARY KEY,
    block_hash           VARCHAR(66)                 NOT NULL,
    extrinsic_index      integer,
    validator_account_id VARCHAR(66)                 NOT NULL,
    amount               VARCHAR(128)                NOT NULL,
    last_updated         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT event_slashed_u_block_hash_rewardee
        UNIQUE (block_hash, validator_account_id),
    CONSTRAINT event_slashed_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT event_slashed_fk_validator_account
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX event_slashed_idx_validator_account_id
    ON event_slashed (validator_account_id);