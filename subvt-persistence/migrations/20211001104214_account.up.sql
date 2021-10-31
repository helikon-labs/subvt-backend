CREATE TABLE IF NOT EXISTS account
(
    id                       VARCHAR(66) PRIMARY KEY,
    discovered_at_block_hash VARCHAR(66),
    killed_at_block_hash     VARCHAR(66),
    last_updated             TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX account_idx_discovered_at_block_hash
    ON account (discovered_at_block_hash);

CREATE INDEX account_idx_killed_at_block_hash
    ON account (killed_at_block_hash);
