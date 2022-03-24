CREATE TABLE IF NOT EXISTS sub_event_democracy_cancelled
(
    id                  SERIAL PRIMARY KEY,
    block_hash          VARCHAR(66) NOT NULL,
    extrinsic_index     integer,
    event_index         integer NOT NULL,
    referendum_index    bigint NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_event_democracy_cancelled
    ADD CONSTRAINT sub_event_democracy_cancelled_fk_block
    FOREIGN KEY (block_hash)
        REFERENCES sub_block (hash)
        ON DELETE CASCADE
        ON UPDATE CASCADE;

CREATE INDEX sub_event_democracy_cancelled_idx_block_hash
    ON sub_event_democracy_cancelled (block_hash);
CREATE INDEX sub_event_democracy_cancelled_idx_referendum_index
    ON sub_event_democracy_cancelled (referendum_index);