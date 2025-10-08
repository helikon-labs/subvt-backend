CREATE TABLE IF NOT EXISTS sub_block
(
    hash                VARCHAR(66) PRIMARY KEY,
    chain_type          VARCHAR(64) NOT NULL,
    number              bigint NOT NULL,
    timestamp           bigint NOT NULL,
    author_account_id   VARCHAR(66),
    era_index           bigint NOT NULL,
    epoch_index         bigint NOT NULL,
    parent_hash         VARCHAR(66) NOT NULL,
    state_root          VARCHAR(66) NOT NULL,
    extrinsics_root     VARCHAR(66) NOT NULL,
    is_finalized        BOOLEAN NOT NULL DEFAULT FALSE,
    metadata_version    smallint NOT NULL,
    runtime_version     smallint NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_block_fk_account
        FOREIGN KEY (author_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE RESTRICT,
    CONSTRAINT sub_block_fk_era
        FOREIGN KEY (era_index)
            REFERENCES sub_era (index)
            ON DELETE RESTRICT
            ON UPDATE RESTRICT

);

CREATE INDEX IF NOT EXISTS sub_block_idx_epoch_index
    ON sub_block (epoch_index);
CREATE INDEX IF NOT EXISTS sub_block_idx_number_chain_type
    ON sub_block (number, chain_type);
CREATE INDEX IF NOT EXISTS sub_block_idx_era_index
    ON sub_block (era_index);
CREATE INDEX IF NOT EXISTS sub_block_idx_number
    ON sub_block (number);
CREATE INDEX IF NOT EXISTS sub_block_idx_timestamp
    ON sub_block (timestamp);
CREATE INDEX IF NOT EXISTS sub_block_idx_hash_epoch_index
    ON sub_block (hash, epoch_index);
CREATE INDEX IF NOT EXISTS sub_block_idx_author_account_id
    ON sub_block (author_account_id);
CREATE INDEX IF NOT EXISTS sub_block_idx_era_index_author_account_id
    ON sub_block (era_index, author_account_id);
CREATE INDEX IF NOT EXISTS sub_block_idx_author_account_id_epoch_index
    ON sub_block (author_account_id, epoch_index);