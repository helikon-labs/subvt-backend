CREATE TABLE IF NOT EXISTS sub_event_slashed
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         integer,
    event_index             integer NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    amount                  VARCHAR(128) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_slashed_u_block_hash_rewardee
        UNIQUE (block_hash, validator_account_id),
    CONSTRAINT sub_event_slashed_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_slashed_fk_validator_account
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX sub_event_slashed_idx_validator_account_id
    ON sub_event_slashed (validator_account_id);

CREATE INDEX sub_event_slashed_idx_block_hash
    ON sub_event_slashed (block_hash);