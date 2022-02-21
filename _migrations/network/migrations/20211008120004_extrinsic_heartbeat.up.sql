CREATE TABLE IF NOT EXISTS sub_extrinsic_heartbeat
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         integer NOT NULL,
    is_nested_call          boolean NOT NULL,
    block_number            bigint NOT NULL,
    session_index           bigint NOT NULL,
    validator_index         bigint NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    is_successful           boolean NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_extrinsic_heartbeat
    ADD CONSTRAINT sub_extrinsic_heartbeat_fk_block
    FOREIGN KEY (block_hash)
        REFERENCES sub_block (hash)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_extrinsic_heartbeat
    ADD CONSTRAINT sub_extrinsic_heartbeat_fk_validator_account_id
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;