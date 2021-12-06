CREATE TABLE IF NOT EXISTS event_era_paid
(
    id               SERIAL PRIMARY KEY,
    block_hash       VARCHAR(66)                    NOT NULL,
    extrinsic_index  integer,
    era_index        bigint                         NOT NULL,
    validator_payout VARCHAR(128)                   NOT NULL,
    remainder        VARCHAR(128)                   NOT NULL,
    last_updated     TIMESTAMP WITHOUT TIME ZONE    NOT NULL DEFAULT now(),
    CONSTRAINT event_era_paid_u_era_index
        UNIQUE (era_index),
    CONSTRAINT event_era_paid_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT event_era_paid_fk_era
        FOREIGN KEY (era_index)
            REFERENCES era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
