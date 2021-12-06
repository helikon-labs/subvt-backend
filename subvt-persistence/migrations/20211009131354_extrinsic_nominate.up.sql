CREATE TABLE IF NOT EXISTS extrinsic_nominate
(
    id                   SERIAL PRIMARY KEY,
    block_hash           VARCHAR(66)                    NOT NULL,
    extrinsic_index      integer                        NOT NULL,
    is_nested_call       boolean                        NOT NULL,
    nominator_account_id VARCHAR(66)                    NOT NULL,
    is_successful        boolean                        NOT NULL,
    last_updated         TIMESTAMP WITHOUT TIME ZONE    NOT NULL DEFAULT now(),
    CONSTRAINT extrinsic_nominate_u_block_hash_nominator
        UNIQUE (block_hash, nominator_account_id),
    CONSTRAINT extrinsic_nominate_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT extrinsic_nominate_fk_nominator
        FOREIGN KEY (nominator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
