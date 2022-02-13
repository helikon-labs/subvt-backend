CREATE TABLE IF NOT EXISTS sub_era_staker
(
    id                      SERIAL PRIMARY KEY,
    era_index               bigint NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    nominator_account_id    VARCHAR(66) NOT NULL,
    stake                   VARCHAR(128) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_era_staker
    ADD CONSTRAINT sub_era_staker_fk_era
    FOREIGN KEY (era_index)
        REFERENCES sub_era (index)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_era_staker
    ADD CONSTRAINT sub_era_staker_fk_validator
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;
ALTER TABLE sub_era_staker
    ADD CONSTRAINT sub_era_staker_fk_nominator
    FOREIGN KEY (nominator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;