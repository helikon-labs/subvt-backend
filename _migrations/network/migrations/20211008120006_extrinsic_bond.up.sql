CREATE TABLE IF NOT EXISTS sub_extrinsic_bond
(
    id                              SERIAL PRIMARY KEY,
    block_hash                      VARCHAR(66) NOT NULL,
    extrinsic_index                 integer NOT NULL,
    is_nested_call                  boolean NOT NULL,
    nesting_index                   text,
    stash_account_id                VARCHAR(66) NOT NULL,
    controller_account_id           VARCHAR(66) NOT NULL,
    amount                          VARCHAR(128) NOT NULL,
    reward_destination_encoded_hex  text NOT NULL,
    is_successful                   boolean NOT NULL,
    created_at                      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_extrinsic_bond_u_extrinsic
        UNIQUE (block_hash, extrinsic_index, nesting_index),
    CONSTRAINT sub_extrinsic_bond_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_extrinsic_bond_fk_stash_account
        FOREIGN KEY (stash_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT sub_extrinsic_bond_fk_controller_account
        FOREIGN KEY (controller_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_extrinsic_bond_idx_block_hash
    ON sub_extrinsic_bond (block_hash);
CREATE INDEX IF NOT EXISTS sub_extrinsic_bond_idx_stash_account_id
    ON sub_extrinsic_bond (stash_account_id);
CREATE INDEX IF NOT EXISTS sub_extrinsic_bond_idx_controller_account_id
    ON sub_extrinsic_bond (controller_account_id);
CREATE INDEX IF NOT EXISTS sub_extrinsic_bond_idx_caller_controller
    ON sub_extrinsic_bond (stash_account_id, controller_account_id);