CREATE TABLE IF NOT EXISTS account
(
    id            VARCHAR(64) PRIMARY KEY,
    discovered_at bigint,
    killed_at     bigint,
    last_updated  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);
