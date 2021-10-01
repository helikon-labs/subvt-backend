CREATE TABLE IF NOT EXISTS account
(
    id            VARCHAR(64) PRIMARY KEY,
    parent_id     VARCHAR(64),
    discovered_at bigint,
    last_updated  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);
