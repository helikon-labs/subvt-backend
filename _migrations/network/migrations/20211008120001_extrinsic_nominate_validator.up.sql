CREATE TABLE IF NOT EXISTS sub_extrinsic_nominate_validator
(
    extrinsic_nominate_id   SERIAL,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_extrinsic_nominate_validator_u_validator
        UNIQUE (extrinsic_nominate_id, validator_account_id),
    CONSTRAINT sub_extrinsic_nominate_validator_fk_extrinsic_nominate
        FOREIGN KEY (extrinsic_nominate_id)
            REFERENCES sub_extrinsic_nominate (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_extrinsic_nominate_validator_fk_validator
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_extrinsic_nominate_validator_idx_validator_account_id
    ON sub_extrinsic_nominate_validator (validator_account_id);