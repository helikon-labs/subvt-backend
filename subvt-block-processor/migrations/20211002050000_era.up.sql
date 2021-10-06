CREATE TABLE IF NOT EXISTS era
(
    index               bigint PRIMARY KEY,
    start_timestamp     bigint                      NOT NULL,
    end_timestamp       bigint                      NOT NULL,
    reward_points_total bigint,
    last_updated        TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);
