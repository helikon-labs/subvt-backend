CREATE TABLE IF NOT EXISTS sub_para_core_assignment
(
    id                          SERIAL PRIMARY KEY,
    block_hash                  VARCHAR(66) NOT NULL,
    para_core_index             bigint NOT NULL,
    para_id                     bigint NOT NULL,
    para_assignment_kind        VARCHAR(128) NOT NULL,
    para_validator_group_index  bigint NOT NULL,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_para_core_assignment_u_block_hash_para_core_index
            UNIQUE (block_hash, para_core_index),
    CONSTRAINT sub_para_core_assignment_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_para_core_assignment_idx_block_hash
    ON sub_para_core_assignment (block_hash);
