CREATE TABLE IF NOT EXISTS sub_era_staker
(
    id                      SERIAL PRIMARY KEY,
    era_index               bigint NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    nominator_account_id    VARCHAR(66) NOT NULL,
    stake                   VARCHAR(128) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_era_staker_u_era_index_validator_nominator
        UNIQUE (era_index, validator_account_id, nominator_account_id),
    CONSTRAINT sub_era_staker_fk_era
        FOREIGN KEY (era_index)
            REFERENCES sub_era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_era_staker_fk_validator
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_era_staker_fk_nominator
            FOREIGN KEY (nominator_account_id)
                REFERENCES sub_account (id)
                ON DELETE CASCADE
                ON UPDATE CASCADE
);

CREATE INDEX sub_era_staker_idx_era_index
    ON sub_era_staker (era_index);

CREATE INDEX sub_era_staker_idx_validator_account_id
    ON sub_era_staker (validator_account_id);

CREATE INDEX sub_era_staker_idx_nominator_account_id
    ON sub_era_staker (nominator_account_id);

CREATE INDEX sub_era_staker_idx_era_index_validator_account_id
    ON sub_era_staker (era_index, validator_account_id);