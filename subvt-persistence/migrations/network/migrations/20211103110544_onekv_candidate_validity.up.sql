CREATE TABLE IF NOT EXISTS sub_onekv_candidate_validity
(
    id                      SERIAL PRIMARY KEY,
    onekv_id                VARCHAR(128) NOT NULL,
    onekv_candidate_id      integer NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    details                 TEXT NOT NULL,
    is_valid                boolean NOT NULL,
    ty                      VARCHAR(128) NOT NULL,
    validity_updated_at     bigint NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_onekv_candidate_validity_fk_one_kv_candidate
        FOREIGN KEY (onekv_candidate_id)
            REFERENCES sub_onekv_candidate (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_onekv_candidate_validity_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX sub_onekv_candidate_validity_idx_one_kv_candidate_id
    ON sub_onekv_candidate_validity (onekv_candidate_id);

CREATE INDEX sub_onekv_candidate_validity_idx_validator_account_id
    ON sub_onekv_candidate_validity (validator_account_id);

CREATE INDEX sub_onekv_candidate_validity_idx_one_kv_candidate_id_is_valid
    ON sub_onekv_candidate_validity (onekv_candidate_id, is_valid);
