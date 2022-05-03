CREATE TABLE IF NOT EXISTS sub_event_democracy_seconded
(
    id              SERIAL PRIMARY KEY,
    block_hash      VARCHAR(66) NOT NULL,
    extrinsic_index integer,
    batch_index     text,
    event_index     integer NOT NULL,
    account_id      VARCHAR(66) NOT NULL,
    proposal_index  bigint NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_democracy_seconded_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_democracy_seconded_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_democracy_seconded_fk_account_id
        FOREIGN KEY (account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_democracy_seconded_idx_block_hash
    ON sub_event_democracy_seconded (block_hash);
CREATE INDEX IF NOT EXISTS sub_event_democracy_seconded_idx_proposal_index
    ON sub_event_democracy_seconded (proposal_index);
CREATE INDEX IF NOT EXISTS sub_event_democracy_seconded_idx_account_id
    ON sub_event_democracy_seconded (account_id);