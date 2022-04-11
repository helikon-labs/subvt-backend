CREATE TABLE IF NOT EXISTS sub_event_validator_offline
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    event_index             integer,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_event_validator_offline
    ADD CONSTRAINT sub_event_validator_offline_u_extrinsic
    UNIQUE (block_hash, event_index);

ALTER TABLE sub_event_validator_offline
    ADD CONSTRAINT sub_event_validator_offline_fk_block
    FOREIGN KEY (block_hash)
        REFERENCES sub_block (hash)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_event_validator_offline
    ADD CONSTRAINT im_online_some_offline_fk_account
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;

CREATE INDEX sub_event_validator_offline_idx_block_hash
    ON sub_event_validator_offline (block_hash);
CREATE INDEX sub_event_validator_offline_idx_validator_account_id
    ON sub_event_validator_offline (validator_account_id);