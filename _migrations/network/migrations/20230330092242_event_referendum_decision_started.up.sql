CREATE TABLE IF NOT EXISTS sub_event_referendum_decision_started
(
    id                  SERIAL PRIMARY KEY,
    block_hash          VARCHAR(66) NOT NULL,
    extrinsic_index     INTEGER,
    nesting_index       TEXT,
    event_index         INTEGER NOT NULL,
    referendum_index    INTEGER NOT NULL,
    track_id            INTEGER NOT NULL,
    ayes                VARCHAR(128) NOT NULL,
    nays                VARCHAR(128) NOT NULL,
    support             VARCHAR(128) NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_referendum_decision_started_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_referendum_decision_started_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_referendum_decision_started_idx_block_hash
    ON sub_event_referendum_decision_started (block_hash);
CREATE INDEX IF NOT EXISTS sub_event_referendum_decision_started_idx_referendum_index
    ON sub_event_referendum_decision_started (referendum_index);