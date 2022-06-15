CREATE TABLE IF NOT EXISTS sub_onekv_candidate_rank_event
(
    onekv_id                VARCHAR(128) PRIMARY KEY,
    validator_account_id    VARCHAR(66) NOT NULL,
    active_era              INTEGER NOT NULL,
    start_era               INTEGER NOT NULL,
    happened_at             bigint NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_onekv_candidate_rank_event_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_onekv_candidate_rank_event_idx_validator_account_id
    ON sub_onekv_candidate_rank_event (validator_account_id);