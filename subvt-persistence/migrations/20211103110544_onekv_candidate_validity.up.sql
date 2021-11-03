CREATE TABLE IF NOT EXISTS onekv_candidate_validity
(
    id                      SERIAL PRIMARY KEY,
    onekv_id                VARCHAR(128)            NOT NULL,
    onekv_candidate_id      SERIAL                  NOT NULL,
    validator_account_id    VARCHAR(66)             NOT NULL,
    details                 TEXT                    NOT NULL,
    is_valid                boolean                 NOT NULL,
    ty                      VARCHAR(128)            NOT NULL,
    updated_at              bigint                  NOT NULL,
    CONSTRAINT onekv_candidate_validity_fk_one_kv_candidate
        FOREIGN KEY (onekv_candidate_id)
            REFERENCES onekv_candidate (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT onekv_candidate_validity_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX onekv_candidate_validity_idx_one_kv_candidate_id
    ON onekv_candidate_validity (onekv_candidate_id);

CREATE INDEX onekv_candidate_validity_idx_validator_account_id
    ON onekv_candidate_validity (validator_account_id);

CREATE INDEX onekv_candidate_validity_idx_one_kv_candidate_id_is_valid
    ON onekv_candidate_validity (onekv_candidate_id, is_valid);
