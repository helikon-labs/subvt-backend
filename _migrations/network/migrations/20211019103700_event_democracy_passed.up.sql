CREATE TABLE IF NOT EXISTS sub_event_democracy_passed
(
    id                  SERIAL PRIMARY KEY,
    block_hash          VARCHAR(66) NOT NULL,
    extrinsic_index     integer,
    batch_index         text,
    event_index         integer NOT NULL,
    referendum_index    bigint NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_democracy_passed_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_democracy_passed_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_democracy_passed_idx_block_hash
    ON sub_event_democracy_passed (block_hash);
CREATE INDEX IF NOT EXISTS sub_event_democracy_passed_idx_referendum_index
    ON sub_event_democracy_passed (referendum_index);