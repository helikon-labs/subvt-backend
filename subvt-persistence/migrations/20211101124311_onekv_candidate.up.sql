CREATE TABLE IF NOT EXISTS onekv_candidate
(
    id                          SERIAL PRIMARY KEY,
    validator_account_id        VARCHAR(66)                 NOT NULL,
    kusama_account_id           VARCHAR(66),
    name                        TEXT                        NOT NULL,
    rank                        bigint,
    is_processed_by_notifier    boolean                     NOT NULL DEFAULT false,
    last_updated                TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT onekv_candidate_fk_validator_account_id
        FOREIGN KEY (validator_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX onekv_candidate_idx_validator_account_id
    ON onekv_candidate (validator_account_id);
