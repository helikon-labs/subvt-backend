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
