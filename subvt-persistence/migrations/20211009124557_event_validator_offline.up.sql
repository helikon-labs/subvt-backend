CREATE TABLE IF NOT EXISTS sub_event_validator_offline
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    event_index             integer,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_validator_offline_u_block_hash_account_id
        UNIQUE (block_hash, validator_account_id),
    CONSTRAINT sub_event_validator_offline_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT im_online_some_offline_fk_account
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX sub_event_validator_offline_idx_validator_account_id
    ON sub_event_validator_offline (validator_account_id);
