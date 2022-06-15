CREATE TABLE IF NOT EXISTS sub_event_democracy_delegated
(
    id                  SERIAL PRIMARY KEY,
    block_hash          VARCHAR(66) NOT NULL,
    extrinsic_index     INTEGER,
    nesting_index       text,
    event_index         INTEGER NOT NULL,
    original_account_id VARCHAR(66) NOT NULL,
    delegate_account_id VARCHAR(66) NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_democracy_delegated_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_democracy_delegated_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_democracy_delegated_fk_original_account
        FOREIGN KEY (original_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_democracy_delegated_fk_delegate_account
        FOREIGN KEY (delegate_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_democracy_delegated_idx_block_hash
    ON sub_event_democracy_delegated (block_hash);
CREATE INDEX IF NOT EXISTS sub_event_democracy_delegated_idx_original_account
    ON sub_event_democracy_delegated (original_account_id);
CREATE INDEX IF NOT EXISTS sub_event_democracy_delegated_idx_delegate_account
    ON sub_event_democracy_delegated (delegate_account_id);