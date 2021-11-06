CREATE TABLE IF NOT EXISTS event_heartbeat_received
(
    block_hash           VARCHAR(66)                 NOT NULL,
    extrinsic_index      integer,
    authority_id         VARCHAR(66)                 NOT NULL,
    last_updated         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT event_heartbeat_received_u_block_hash_authority_id
        UNIQUE (block_hash, authority_id),
    CONSTRAINT event_heartbeat_received_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX event_heartbeat_received_idx_authority_id
    ON event_heartbeat_received (authority_id);
