CREATE TABLE IF NOT EXISTS sub_extrinsic_nominate_validator
(
    extrinsic_nominate_id   SERIAL,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_extrinsic_nominate_validator
    ADD CONSTRAINT sub_extrinsic_nominate_validator_u_validator
    UNIQUE (extrinsic_nominate_id, validator_account_id);

ALTER TABLE sub_extrinsic_nominate_validator
    ADD CONSTRAINT sub_extrinsic_nominate_validator_fk_extrinsic_nominate
    FOREIGN KEY (extrinsic_nominate_id)
        REFERENCES sub_extrinsic_nominate (id)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_extrinsic_nominate_validator
    ADD CONSTRAINT sub_extrinsic_nominate_validator_fk_validator
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;