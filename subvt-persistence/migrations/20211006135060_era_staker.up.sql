CREATE TABLE IF NOT EXISTS era_staker
(
    id                              SERIAL PRIMARY KEY,
    era_index                       bigint                      NOT NULL,
    validator_account_id            VARCHAR(66)                 NOT NULL,
    nominator_account_id            VARCHAR(66)                 NOT NULL,
    stake                           VARCHAR(128)                NOT NULL,
    last_updated                    TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT era_staker_u_era_index_validator_nominator
        UNIQUE (era_index, validator_account_id, nominator_account_id),
    CONSTRAINT era_staker_fk_era
        FOREIGN KEY (era_index)
            REFERENCES era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT era_staker_fk_validator
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT era_staker_fk_nominator
            FOREIGN KEY (nominator_account_id)
                REFERENCES account (id)
                ON DELETE CASCADE
                ON UPDATE CASCADE
);

CREATE INDEX era_staker_idx_era_index
    ON era_staker (era_index);

CREATE INDEX era_staker_idx_validator_account_id
    ON era_staker (validator_account_id);

CREATE INDEX era_staker_idx_nominator_account_id
    ON era_staker (nominator_account_id);

CREATE INDEX era_staker_idx_era_index_validator_account_id
    ON era_staker (era_index, validator_account_id);