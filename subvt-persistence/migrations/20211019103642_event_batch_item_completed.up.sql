CREATE TABLE IF NOT EXISTS event_batch_item_completed
(
    id              SERIAL PRIMARY KEY,
    block_hash      VARCHAR(66)                 NOT NULL,
    extrinsic_index integer,
    last_updated    TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT event_batch_item_completed_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
