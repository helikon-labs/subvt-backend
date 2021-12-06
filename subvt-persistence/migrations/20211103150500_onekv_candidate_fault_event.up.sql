CREATE TABLE IF NOT EXISTS onekv_candidate_fault_event
(
    onekv_id                VARCHAR(128) PRIMARY KEY,
    validator_account_id    VARCHAR(66)                 NOT NULL,
    previous_rank           integer,
    reason                  TEXT,
    happened_at             bigint                      NOT NULL,
    last_updated            TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT onekv_candidate_fault_event_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX onekv_candidate_fault_event_idx_validator_account_id
    ON onekv_candidate_fault_event (validator_account_id);
