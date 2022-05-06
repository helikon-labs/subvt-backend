CREATE TABLE IF NOT EXISTS sub_event_payout_started
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         integer,
    nesting_index           text,
    event_index             integer NOT NULL,
    era_index               bigint NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_payout_started_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_payout_started_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_payout_started_fk_era
        FOREIGN KEY (era_index)
            REFERENCES sub_era (index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_payout_started_fk_validator
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_payout_started_idx_block_hash
    ON sub_event_payout_started (block_hash);
CREATE INDEX IF NOT EXISTS sub_event_payout_started_idx_validator
    ON sub_event_payout_started (validator_account_id);
CREATE INDEX IF NOT EXISTS sub_event_payout_started_idx_era_validator
    ON sub_event_payout_started (era_index, validator_account_id);