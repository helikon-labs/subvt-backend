CREATE TABLE IF NOT EXISTS event
(
    id           SERIAL PRIMARY KEY,
    block_hash   VARCHAR(64) NOT NULL,
    module_index smallint NOT NULL,
    event_index  smallint NOT NULL,
    module_name  VARCHAR(2048) NOT NULL,
    event_name   VARCHAR(2048) NOT NULL,
    last_updated TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT u_event
        UNIQUE (block_hash, module_index, event_index),
    CONSTRAINT fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
