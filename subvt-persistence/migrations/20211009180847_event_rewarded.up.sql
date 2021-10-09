CREATE TABLE IF NOT EXISTS event_rewarded
(
    id                  SERIAL PRIMARY KEY,
    block_hash          VARCHAR(66)                 NOT NULL,
    extrinsic_index     integer,
    rewardee_account_id VARCHAR(66)                 NOT NULL,
    amount              VARCHAR(128)                NOT NULL,
    last_updated        TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT event_rewarded_u_block_hash_rewardee
        UNIQUE (block_hash, rewardee_account_id),
    CONSTRAINT event_rewarded_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT event_rewarded_fk_rewardee
        FOREIGN KEY (rewardee_account_id)
            REFERENCES account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);
