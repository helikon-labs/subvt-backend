CREATE TABLE IF NOT EXISTS sub_telemetry_network_status (
    id                      INTEGER PRIMARY KEY,
    best_block_number       bigint NOT NULL,
    best_block_timestamp    bigint NOT NULL,
    average_block_time_ms   bigint,
    finalized_block_number  bigint NOT NULL,
    finalized_block_hash    VARCHAR(66) NOT NULL,
    updated_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

INSERT INTO sub_telemetry_network_status(id, best_block_number, best_block_timestamp, average_block_time_ms, finalized_block_number, finalized_block_hash)
VALUES(1, 0, 0, NULL, 0, '')
ON CONFLICT (id) DO NOTHING;