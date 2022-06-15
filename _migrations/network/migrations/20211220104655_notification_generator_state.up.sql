CREATE TABLE IF NOT EXISTS sub_notification_generator_state
(
    id                          INTEGER PRIMARY KEY,
    last_processed_block_hash   VARCHAR(66) NOT NULL,
    last_processed_block_number bigint NOT NULL,
    updated_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);