CREATE TABLE IF NOT EXISTS sub_event_democracy_proposed
(
    id              SERIAL PRIMARY KEY,
    block_hash      VARCHAR(66) NOT NULL,
    extrinsic_index INTEGER,
    nesting_index   text,
    event_index     INTEGER NOT NULL,
    proposal_index  bigint NOT NULL,
    deposit         VARCHAR(128) NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_democracy_proposed_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_democracy_proposed_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_democracy_proposed_idx_block_hash
    ON sub_event_democracy_undelegated (block_hash);
CREATE INDEX IF NOT EXISTS sub_event_democracy_proposed_idx_proposal_index
    ON sub_event_democracy_proposed (proposal_index);