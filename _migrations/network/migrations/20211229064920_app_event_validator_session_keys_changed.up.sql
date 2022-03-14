CREATE TABLE IF NOT EXISTS sub_app_event_validator_session_keys_changed
(
    id                          SERIAL PRIMARY KEY,
    validator_account_id        VARCHAR(66) NOT NULL,
    session_keys                text NOT NULL,
    discovered_block_number     bigint NOT NULL,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_app_event_validator_session_keys_changed
    ADD CONSTRAINT sub_app_event_validator_session_keys_changed_fk_validator
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;

CREATE INDEX sub_app_event_session_keys_changed_idx_validator
    ON sub_app_event_validator_session_keys_changed (validator_account_id);