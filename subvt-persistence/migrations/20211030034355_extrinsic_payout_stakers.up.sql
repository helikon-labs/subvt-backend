CREATE TABLE IF NOT EXISTS extrinsic_payout_stakers
(
    id                   SERIAL PRIMARY KEY,
    block_hash           VARCHAR(66)                    NOT NULL,
    extrinsic_index      integer                        NOT NULL,
    is_nested_call       boolean                        NOT NULL,
    caller_account_id    VARCHAR(66)                    NOT NULL,
    validator_account_id VARCHAR(66)                    NOT NULL,
    era_index            bigint                         NOT NULL,
    is_successful        boolean                        NOT NULL,
    last_updated         TIMESTAMP WITHOUT TIME ZONE    NOT NULL DEFAULT now(),
    CONSTRAINT extrinsic_payout_stakers_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT extrinsic_payout_stakers_fk_caller_account_id
        FOREIGN KEY (caller_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT extrinsic_payout_stakers_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT extrinsic_payout_stakers_fk_era
        FOREIGN KEY (era_index)
            REFERENCES era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX extrinsic_payout_stakers_idx_caller_account_id
    ON extrinsic_payout_stakers (caller_account_id);

CREATE INDEX extrinsic_payout_stakers_idx_validator_account_id
    ON extrinsic_payout_stakers (validator_account_id);

CREATE INDEX extrinsic_payout_stakers_idx_era_index
    ON extrinsic_payout_stakers (era_index);

CREATE INDEX extrinsic_payout_stakers_idx_is_successful
    ON extrinsic_payout_stakers (is_successful);

CREATE INDEX extrinsic_payout_stakers_idx_validator_era_successful
    ON extrinsic_payout_stakers (validator_account_id, era_index, is_successful);

CREATE INDEX extrinsic_payout_stakers_idx_index_era_index_block_hash_success
    ON extrinsic_payout_stakers (era_index, extrinsic_index, block_hash, is_successful);
