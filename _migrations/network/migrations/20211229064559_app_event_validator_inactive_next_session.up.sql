CREATE TABLE IF NOT EXISTS sub_app_event_validator_inactive_next_session
(
    id                          SERIAL PRIMARY KEY,
    validator_account_id        VARCHAR(66) NOT NULL,
    discovered_block_number     bigint NOT NULL,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_app_event_validator_inactive_next_session
    ADD CONSTRAINT sub_app_event_validator_inactive_next_session_fk_validator
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;

CREATE INDEX sub_app_event_validator_inactive_next_session_idx_validator
    ON sub_app_event_validator_inactive_next_session (validator_account_id);