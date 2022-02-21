CREATE TABLE IF NOT EXISTS sub_era_validator
(
    id                      SERIAL PRIMARY KEY,
    era_index               bigint NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    controller_account_id   VARCHAR(66),
    is_active               boolean NOT NULL DEFAULT false,
    active_validator_index  bigint,
    commission_per_billion  bigint,
    blocks_nominations      bool,
    self_stake              VARCHAR(128),
    total_stake             VARCHAR(128),
    reward_points           bigint NOT NULL DEFAULT 0,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_era_validator
    ADD CONSTRAINT sub_era_validator_u_era_index_validator
    UNIQUE (era_index, validator_account_id);

ALTER TABLE sub_era_validator
    ADD CONSTRAINT sub_era_validator_fk_era
    FOREIGN KEY (era_index)
        REFERENCES sub_era (index)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_era_validator
    ADD CONSTRAINT sub_era_validator_fk_account
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;
ALTER TABLE sub_era_validator
    ADD CONSTRAINT sub_era_validator_fk_controller_account
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;