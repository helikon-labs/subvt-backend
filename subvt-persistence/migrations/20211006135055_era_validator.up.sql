CREATE TABLE IF NOT EXISTS era_validator
(
    id                   SERIAL PRIMARY KEY,
    era_index            bigint                      NOT NULL,
    validator_account_id VARCHAR(66)                 NOT NULL,
    is_active            boolean                     NOT NULL DEFAULT false,
    reward_points        bigint                      NOT NULL DEFAULT 0,
    last_updated         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT era_validator_u_era_index_validator_account_id
        UNIQUE (era_index, validator_account_id),
    CONSTRAINT era_validator_fk_era
        FOREIGN KEY (era_index)
            REFERENCES era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT era_validator_fk_account
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX era_validator_era_index_validator_account_id
ON era_validator (era_index, validator_account_id);
