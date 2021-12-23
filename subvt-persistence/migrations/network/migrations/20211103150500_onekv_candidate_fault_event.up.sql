CREATE TABLE IF NOT EXISTS sub_onekv_candidate_fault_event
(
    onekv_id                VARCHAR(128) PRIMARY KEY,
    validator_account_id    VARCHAR(66) NOT NULL,
    previous_rank           integer,
    reason                  TEXT,
    happened_at             bigint NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_onekv_candidate_fault_event_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX sub_onekv_candidate_fault_event_idx_validator_account_id
    ON sub_onekv_candidate_fault_event (validator_account_id);
