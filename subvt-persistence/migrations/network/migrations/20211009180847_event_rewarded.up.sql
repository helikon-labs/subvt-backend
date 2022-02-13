CREATE TABLE IF NOT EXISTS sub_event_rewarded
(
    id                  SERIAL PRIMARY KEY,
    block_hash          VARCHAR(66) NOT NULL,
    extrinsic_index     integer,
    event_index         integer NOT NULL,
    rewardee_account_id VARCHAR(66) NOT NULL,
    amount              VARCHAR(128) NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_event_rewarded
    ADD CONSTRAINT sub_event_rewarded_fk_block
    FOREIGN KEY (block_hash)
        REFERENCES sub_block (hash)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_event_rewarded
    ADD CONSTRAINT sub_event_rewarded_fk_rewardee
    FOREIGN KEY (rewardee_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;