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
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT block_fk_era
        FOREIGN KEY (era_index)
            REFERENCES era (index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT block_fk_epoch
        FOREIGN KEY (epoch_index)
            REFERENCES epoch (index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);
