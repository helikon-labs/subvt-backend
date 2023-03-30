CREATE TABLE IF NOT EXISTS sub_event_referendum_submitted
(
    id                  SERIAL PRIMARY KEY,
    block_hash          VARCHAR(66) NOT NULL,
    extrinsic_index     INTEGER,
    nesting_index       TEXT,
    event_index         INTEGER NOT NULL,
    referendum_index    INTEGER NOT NULL,
    track_id            INTEGER NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_referendum_submitted_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_referendum_submitted_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_referendum_submitted_idx_block_hash
    ON sub_event_referendum_submitted (block_hash);
CREATE INDEX IF NOT EXISTS sub_event_referendum_submitted_idx_referendum_index
    ON sub_event_referendum_submitted (referendum_index);