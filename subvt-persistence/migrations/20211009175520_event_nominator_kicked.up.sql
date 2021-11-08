CREATE TABLE IF NOT EXISTS event_nominator_kicked
(
    id                   SERIAL PRIMARY KEY,
    block_hash           VARCHAR(66)                 NOT NULL,
    extrinsic_index      integer,
    validator_account_id VARCHAR(66)                 NOT NULL,
    nominator_account_id VARCHAR(66)                 NOT NULL,
    last_updated         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT event_nominator_kicked_u_block_hash_validator_nominator
        UNIQUE (block_hash, validator_account_id, nominator_account_id),
    CONSTRAINT event_nominator_kicked_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT event_nominator_kicked_fk_validator
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT event_nominator_kicked_fk_nominator
        FOREIGN KEY (nominator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
