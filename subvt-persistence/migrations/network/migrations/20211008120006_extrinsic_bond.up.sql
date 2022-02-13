CREATE TABLE IF NOT EXISTS sub_extrinsic_bond
(
    id                              SERIAL PRIMARY KEY,
    block_hash                      VARCHAR(66) NOT NULL,
    extrinsic_index                 integer NOT NULL,
    is_nested_call                  boolean NOT NULL,
    stash_account_id                VARCHAR(66) NOT NULL,
    controller_account_id           VARCHAR(66) NOT NULL,
    amount                          VARCHAR(128) NOT NULL,
    reward_destination_encoded_hex  text NOT NULL,
    is_successful                   boolean NOT NULL,
    created_at                      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_extrinsic_bond
    ADD CONSTRAINT sub_extrinsic_bond_fk_block
    FOREIGN KEY (block_hash)
        REFERENCES sub_block (hash)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_extrinsic_bond
    ADD CONSTRAINT sub_extrinsic_bond_fk_stash_account
    FOREIGN KEY (stash_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;
ALTER TABLE sub_extrinsic_bond
    ADD CONSTRAINT sub_extrinsic_bond_fk_controller_account
    FOREIGN KEY (controller_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;