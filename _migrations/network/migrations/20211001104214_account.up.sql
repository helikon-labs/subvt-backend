CREATE TABLE IF NOT EXISTS sub_account
(
    id                          VARCHAR(66) PRIMARY KEY,
    discovered_at_block_hash    VARCHAR(66),
    discovered_at_block_number  bigint,
    discovered_at               bigint,
    killed_at_block_hash        VARCHAR(66),
    killed_at_block_number      bigint,
    killed_at                   bigint,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS sub_account_idx_discovered_at_block_hash
    ON sub_account (discovered_at_block_hash);
CREATE INDEX IF NOT EXISTS sub_account_idx_id_discovered_at_block_hash
    ON sub_account (id, discovered_at_block_hash);
CREATE INDEX IF NOT EXISTS sub_account_idx_killed_at_block_hash
    ON sub_account (killed_at_block_hash);
CREATE INDEX IF NOT EXISTS sub_account_idx_id_killed_at_block_hash
    ON sub_account (id, killed_at_block_hash);