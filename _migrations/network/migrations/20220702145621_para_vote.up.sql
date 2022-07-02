CREATE TABLE IF NOT EXISTS sub_para_vote
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    session_index           bigint NOT NULL,
    para_id                 bigint NOT NULL,
    para_validator_index    bigint NOT NULL,
    is_explicit             boolean NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_para_vote_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_para_vote_idx_block_hash
    ON sub_para_vote (block_hash);
