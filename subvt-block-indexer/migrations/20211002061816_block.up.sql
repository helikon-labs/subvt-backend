CREATE TABLE IF NOT EXISTS block
(
    hash              VARCHAR(64) PRIMARY KEY,
    number            bigint                      NOT NULL,
    timestamp         bigint                      NOT NULL,
    author_account_id VARCHAR(64)                 NOT NULL,
    parent_hash       VARCHAR(64)                 NOT NULL,
    state_root        VARCHAR(64)                 NOT NULL,
    extrinsics_root   VARCHAR(64)                 NOT NULL,
    finalized         BOOLEAN                     NOT NULL DEFAULT FALSE,
    metadata_version  smallint                    NOT NULL,
    runtime_version   smallint                    NOT NULL,
    last_updated      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT fk_parent
        FOREIGN KEY (parent_hash)
            REFERENCES block (hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT fk_account
        FOREIGN KEY (author_account_id)
            REFERENCES account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);
