CREATE TABLE IF NOT EXISTS sub_extrinsic_payout_stakers
(
    id                      SERIAL PRIMARY KEY,
    block_hash              VARCHAR(66) NOT NULL,
    extrinsic_index         integer NOT NULL,
    is_nested_call          boolean NOT NULL,
    caller_account_id       VARCHAR(66) NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    era_index               bigint NOT NULL,
    is_successful           boolean NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_extrinsic_payout_stakers
    ADD CONSTRAINT sub_extrinsic_payout_stakers_fk_block
    FOREIGN KEY (block_hash)
        REFERENCES sub_block (hash)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_extrinsic_payout_stakers
    ADD CONSTRAINT sub_extrinsic_payout_stakers_fk_caller_account_id
    FOREIGN KEY (caller_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;
ALTER TABLE sub_extrinsic_payout_stakers
    ADD CONSTRAINT sub_extrinsic_payout_stakers_fk_validator_account_id
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;
ALTER TABLE sub_extrinsic_payout_stakers
    ADD CONSTRAINT sub_extrinsic_payout_stakers_fk_era
    FOREIGN KEY (era_index)
        REFERENCES sub_era (index)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;