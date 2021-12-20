CREATE TABLE IF NOT EXISTS sub_notification_generator_state
(
    id                          integer PRIMARY KEY,
    last_processed_block_hash   VARCHAR(66) NOT NULL,
    last_processed_block_number bigint NOT NULL,
    updated_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_notification_generator_state_fk_block
        FOREIGN KEY (last_processed_block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);