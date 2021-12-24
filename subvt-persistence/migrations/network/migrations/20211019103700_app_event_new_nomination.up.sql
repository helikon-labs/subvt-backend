CREATE TABLE IF NOT EXISTS sub_app_event_new_nomination
(
    id                              SERIAL PRIMARY KEY,
    block_hash                      VARCHAR(66) NOT NULL,
    extrinsic_index                 integer NOT NULL,
    nominator_stash_account_id      VARCHAR(66) NOT NULL,
    nominator_controller_account_id VARCHAR(66) NOT NULL,
    validator_account_id            VARCHAR(66) NOT NULL,
    amount                          VARCHAR(128) NOT NULL,
    nominee_count                   integer NOT NULL,
    created_at                      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_app_event_new_nomination_u_block_nominator_validator
        UNIQUE (block_hash, nominator_stash_account_id, validator_account_id),
    CONSTRAINT sub_app_event_new_nomination_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_app_event_new_nomination_fk_nominator_stash
        FOREIGN KEY (nominator_stash_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_app_event_new_nomination_fk_nominator_controller
        FOREIGN KEY (nominator_controller_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_app_event_new_nomination_fk_validator
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX sub_app_event_new_nomination_idx_block_hash
    ON sub_app_event_new_nomination (block_hash);

CREATE INDEX sub_app_event_new_nomination_idx_validator
    ON sub_app_event_new_nomination (validator_account_id);