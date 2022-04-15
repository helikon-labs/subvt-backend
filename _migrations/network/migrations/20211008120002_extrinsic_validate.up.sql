CREATE TABLE IF NOT EXISTS sub_extrinsic_validate
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         integer NOT NULL,
    is_nested_call          boolean NOT NULL,
    stash_account_id        VARCHAR(66) NOT NULL,
    controller_account_id   VARCHAR(66) NOT NULL,
    commission_per_billion  bigint NOT NULL,
    blocks_nominations      boolean NOT NULL,
    is_successful           boolean NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE  NOT NULL DEFAULT now(),
    CONSTRAINT sub_extrinsic_validate_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_extrinsic_validate_fk_stash
        FOREIGN KEY (stash_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT sub_extrinsic_validate_fk_controller
        FOREIGN KEY (controller_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_extrinsic_validate_idx_block_hash
    ON sub_extrinsic_validate (block_hash);