CREATE TABLE IF NOT EXISTS sub_event_nominator_kicked
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         integer,
    batch_index             text,
    event_index             integer NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    nominator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_event_nominator_kicked_u_event
        UNIQUE (block_hash, event_index),
    CONSTRAINT sub_event_nominator_kicked_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES sub_block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_nominator_kicked_fk_validator
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT sub_event_nominator_kicked_fk_nominator
        FOREIGN KEY (nominator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_event_nominator_kicked_idx_block_hash
    ON sub_event_nominator_kicked (block_hash);