CREATE TABLE IF NOT EXISTS epoch
(
    index              bigint PRIMARY KEY,
    era_index          bigint                      NOT NULL,
    start_block_number bigint                      NOT NULL,
    start_timestamp    bigint                      NOT NULL,
    end_timestamp      bigint                      NOT NULL,
    last_updated       TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT fk_era
        FOREIGN KEY (era_index)
            REFERENCES era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
