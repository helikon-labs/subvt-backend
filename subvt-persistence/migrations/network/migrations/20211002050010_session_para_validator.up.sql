CREATE TABLE IF NOT EXISTS sub_session_para_validator
(
    id                      SERIAL PRIMARY KEY,
    era_index               bigint NOT NULL,
    session_index           bigint NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_session_para_validator
    ADD CONSTRAINT sub_session_para_validator_fk_era
    FOREIGN KEY (era_index)
        REFERENCES sub_era (index)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_session_para_validator
    ADD CONSTRAINT sub_session_para_validator_fk_account
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;