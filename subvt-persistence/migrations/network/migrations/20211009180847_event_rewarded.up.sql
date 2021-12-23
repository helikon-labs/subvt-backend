CREATE TABLE IF NOT EXISTS sub_event_rewarded
(
    id                  SERIAL PRIMARY KEY,
    block_hash          VARCHAR(66) NOT NULL,
    extrinsic_index     integer,
    event_index         integer NOT NULL,
    rewardee_account_id VARCHAR(66) NOT NULL,
    amount              VARCHAR(128) NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_rewarded_u_block_hash_rewardee
        UNIQUE (block_hash, rewardee_account_id),
    CONSTRAINT sub_event_rewarded_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_rewarded_fk_rewardee
        FOREIGN KEY (rewardee_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX sub_event_rewarded_idx_block_hash
    ON sub_event_rewarded (block_hash);

CREATE INDEX sub_event_rewarded_idx_rewardee_account_id
    ON sub_event_rewarded (rewardee_account_id);

CREATE INDEX sub_event_rewarded_idx_extrinsic_index_block_hash_rewardee
    ON sub_event_rewarded (extrinsic_index, block_hash, rewardee_account_id);