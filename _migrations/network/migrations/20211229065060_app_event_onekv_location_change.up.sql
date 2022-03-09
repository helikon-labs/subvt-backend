CREATE TABLE IF NOT EXISTS sub_app_event_onekv_location_change
(
    id                          SERIAL PRIMARY KEY,
    validator_account_id        VARCHAR(66) NOT NULL,
    prev_location               text,
    current_location            text,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_app_event_onekv_location_change
    ADD CONSTRAINT sub_app_event_onekv_location_change_fk_validator
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;