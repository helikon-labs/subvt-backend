CREATE TABLE IF NOT EXISTS era
(
    index                   bigint PRIMARY KEY,
    start_timestamp         bigint                      NOT NULL,
    end_timestamp           bigint                      NOT NULL,
    active_nominator_count  bigint                      NOT NULL,
    total_stake             VARCHAR(128)                NOT NULL,
    minimum_stake           VARCHAR(128)                NOT NULL,
    maximum_stake           VARCHAR(128)                NOT NULL,
    average_stake           VARCHAR(128)                NOT NULL,
    median_stake            VARCHAR(128)                NOT NULL,
    reward_points_total     bigint,
    last_updated            TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);
