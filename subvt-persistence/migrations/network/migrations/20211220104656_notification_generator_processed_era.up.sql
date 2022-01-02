CREATE TABLE IF NOT EXISTS sub_notification_generator_processed_era
(
    era_index   bigint PRIMARY KEY,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);