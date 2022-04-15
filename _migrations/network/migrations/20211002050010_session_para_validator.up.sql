CREATE TABLE IF NOT EXISTS sub_session_para_validator
(
    id                      SERIAL PRIMARY KEY,
    era_index               bigint NOT NULL,
    session_index           bigint NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_session_para_validator_u_session_index_validator
        UNIQUE (session_index, validator_account_id),
    CONSTRAINT sub_session_para_validator_fk_era
        FOREIGN KEY (era_index)
            REFERENCES sub_era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_session_para_validator_fk_account
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_session_para_validator_idx_era_index
    ON sub_session_para_validator (session_index);
CREATE INDEX IF NOT EXISTS sub_session_para_validator_idx_session_index
    ON sub_session_para_validator (session_index);
CREATE INDEX IF NOT EXISTS sub_session_para_validator_idx_validator_account_id
    ON sub_session_para_validator (validator_account_id);
CREATE INDEX IF NOT EXISTS sub_session_para_validator_idx_session_index_validator
    ON sub_session_para_validator (session_index, validator_account_id);