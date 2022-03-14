CREATE TABLE IF NOT EXISTS sub_app_event_nomination_amount_change
(
    id                          SERIAL PRIMARY KEY,
    validator_account_id        VARCHAR(66) NOT NULL,
    discovered_block_number     bigint NOT NULL,
    nominator_stash_account_id  VARCHAR(66) NOT NULL,
    prev_active_amount          VARCHAR(128) NOT NULL,
    prev_total_amount           VARCHAR(128) NOT NULL,
    prev_nominee_count          bigint NOT NULL,
    active_amount               VARCHAR(128) NOT NULL,
    total_amount                VARCHAR(128) NOT NULL,
    nominee_count               bigint NOT NULL,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_app_event_nomination_amount_change
    ADD CONSTRAINT sub_app_event_nomination_amount_change_fk_validator
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;
ALTER TABLE sub_app_event_nomination_amount_change
    ADD CONSTRAINT sub_app_event_nomination_amount_change_fk_nominator
    FOREIGN KEY (nominator_stash_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;

CREATE INDEX sub_app_event_nomination_amount_change_idx_validator
    ON sub_app_event_nomination_amount_change (validator_account_id);