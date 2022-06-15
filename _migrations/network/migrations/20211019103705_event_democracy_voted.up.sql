CREATE TABLE IF NOT EXISTS sub_event_democracy_voted
(
    id                  SERIAL PRIMARY KEY,
    block_hash          VARCHAR(66) NOT NULL,
    extrinsic_index     INTEGER,
    nesting_index       text,
    event_index         INTEGER NOT NULL,
    account_id          VARCHAR(66) NOT NULL,
    referendum_index    bigint NOT NULL,
    aye_balance         VARCHAR(128),
    nay_balance         VARCHAR(128),
    conviction          INTEGER,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_democracy_voted_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_democracy_voted_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_democracy_voted_fk_account_id
        FOREIGN KEY (account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_democracy_voted_idx_block_hash
    ON sub_event_democracy_voted (block_hash);
CREATE INDEX IF NOT EXISTS sub_event_democracy_voted_idx_referendum_index
    ON sub_event_democracy_voted (referendum_index);
CREATE INDEX IF NOT EXISTS sub_event_democracy_voted_idx_account_id
    ON sub_event_democracy_voted (account_id);