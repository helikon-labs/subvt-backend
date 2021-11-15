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
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT event_rewarded_fk_rewardee
        FOREIGN KEY (rewardee_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX event_rewarded_idx_block_hash
    ON event_rewarded (block_hash);

CREATE INDEX event_rewarded_idx_rewardee_account_id
    ON event_rewarded (rewardee_account_id);

CREATE INDEX event_rewarded_idx_extrinsic_index_block_hash_rewardee
    ON event_rewarded (extrinsic_index, block_hash, rewardee_account_id);