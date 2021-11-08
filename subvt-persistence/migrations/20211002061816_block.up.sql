CREATE TABLE IF NOT EXISTS block
(
    hash              VARCHAR(66) PRIMARY KEY,
    number            bigint                      NOT NULL,
    timestamp         bigint,
    author_account_id VARCHAR(66),
    era_index         bigint                      NOT NULL,
    epoch_index       bigint                      NOT NULL,
    parent_hash       VARCHAR(66)                 NOT NULL,
    state_root        VARCHAR(66)                 NOT NULL,
    extrinsics_root   VARCHAR(66)                 NOT NULL,
    is_finalized      BOOLEAN                     NOT NULL DEFAULT FALSE,
    metadata_version  smallint                    NOT NULL,
    runtime_version   smallint                    NOT NULL,
    last_updated      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT block_fk_account
        FOREIGN KEY (author_account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT block_fk_era
        FOREIGN KEY (era_index)
            REFERENCES era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

ALTER TABLE account
    ADD CONSTRAINT account_fk_discovered_block
        FOREIGN KEY (discovered_at_block_hash)
            REFERENCES block (hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE;
ALTER TABLE account
    ADD CONSTRAINT account_fk_killed_block
        FOREIGN KEY (killed_at_block_hash)
            REFERENCES block (hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE;

CREATE INDEX block_idx_epoch_index
    ON block (epoch_index);

CREATE INDEX block_idx_era_index
    ON block (era_index);

CREATE INDEX block_idx_number
    ON block (number);

CREATE INDEX block_idx_hash_epoch_index
    ON block (hash, epoch_index);

CREATE INDEX block_idx_author_account_id
    ON block (author_account_id);

CREATE INDEX block_idx_era_index_author_account_id
    ON block (era_index, author_account_id);