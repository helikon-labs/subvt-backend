CREATE TABLE IF NOT EXISTS event_validator_offline
(
    block_hash           VARCHAR(66)                 NOT NULL,
    validator_account_id VARCHAR(66)                 NOT NULL,
    last_updated         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT event_validator_offline_u_block_hash_account_id
        UNIQUE (block_hash, validator_account_id),
    CONSTRAINT event_validator_offline_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT im_online_some_offline_fk_account
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
