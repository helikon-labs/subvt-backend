CREATE TABLE IF NOT EXISTS sub_epoch
(
    index               bigint PRIMARY KEY,
    era_index           bigint NOT NULL,
    start_block_number  bigint NOT NULL DEFAULT 0,
    start_timestamp     bigint NOT NULL DEFAULT 0,
    end_timestamp       bigint NOT NULL DEFAULT 0,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_epoch_fk_era
        FOREIGN KEY (era_index)
            REFERENCES sub_era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_epoch_idx_era
    ON sub_epoch (era_index);