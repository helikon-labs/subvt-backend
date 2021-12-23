CREATE TABLE IF NOT EXISTS sub_extrinsic_set_controller
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         integer NOT NULL,
    is_nested_call          boolean NOT NULL,
    caller_account_id       VARCHAR(66) NOT NULL,
    controller_account_id   VARCHAR(66) NOT NULL,
    is_successful           boolean NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_extrinsic_set_controller_u_block_hash_caller
        UNIQUE (block_hash, caller_account_id),
    CONSTRAINT sub_extrinsic_set_controller_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_extrinsic_set_controller_fk_caller_account
        FOREIGN KEY (caller_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_extrinsic_set_controller_fk_controller_account
        FOREIGN KEY (controller_account_id)
            REFERENCES sub_account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX sub_extrinsic_set_controller_idx_caller_account_id
    ON sub_extrinsic_set_controller (caller_account_id);

CREATE INDEX sub_extrinsic_set_controller_idx_controller_account_id
    ON sub_extrinsic_set_controller (controller_account_id);