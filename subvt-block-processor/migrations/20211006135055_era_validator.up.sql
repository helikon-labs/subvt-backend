CREATE TABLE IF NOT EXISTS era_validator
(
    id                   SERIAL PRIMARY KEY,
    era_index            bigint                      NOT NULL,
    validator_account_id VARCHAR(66)                 NOT NULL,
    is_active            boolean                     NOT NULL DEFAULT false,
    reward_points        bigint                      NOT NULL DEFAULT 0,
    last_updated         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT u_era_validator
        UNIQUE (era_index, validator_account_id),
    CONSTRAINT fk_era
        FOREIGN KEY (era_index)
            REFERENCES era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT fk_account
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
