CREATE TABLE IF NOT EXISTS extrinsic_nominate_validator
(
    extrinsic_nominate_id SERIAL,
    validator_account_id  VARCHAR(66)                 NOT NULL,
    last_updated          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT extrinsic_nominate_validator_fk_extrinsic_nominate
        FOREIGN KEY (extrinsic_nominate_id)
            REFERENCES extrinsic_nominate (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT extrinsic_nominate_validator_u_validator
        UNIQUE (extrinsic_nominate_id, validator_account_id),
    CONSTRAINT extrinsic_nominate_validator_fk_validator
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
