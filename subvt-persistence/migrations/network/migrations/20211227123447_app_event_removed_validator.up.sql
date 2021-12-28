CREATE TABLE IF NOT EXISTS sub_app_event_removed_validator
(
    id                      SERIAL PRIMARY KEY,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_app_event_removed_validator_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX sub_app_event_removed_validator_idx_validator_account_id
    ON sub_app_event_removed_validator (validator_account_id);