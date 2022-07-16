CREATE TABLE IF NOT EXISTS sub_app_event_validator_commission_changed
(
    id                              SERIAL PRIMARY KEY,
    validator_account_id            VARCHAR(66) NOT NULL,
    previous_commission_per_billion bigint NOT NULL,
    current_commission_per_billion  bigint NOT NULL,
    discovered_block_number         bigint NOT NULL,
    created_at                      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_app_event_validator_commission_changed_fk_validator
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_app_event_commission_changed_idx_validator
    ON sub_app_event_validator_commission_changed (validator_account_id);