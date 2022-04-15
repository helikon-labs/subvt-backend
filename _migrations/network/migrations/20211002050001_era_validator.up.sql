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
    active_nominator_count  integer,
    reward_points           bigint NOT NULL DEFAULT 0,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_era_validator_u_era_index_validator
        UNIQUE (era_index, validator_account_id),
    CONSTRAINT sub_era_validator_fk_era
        FOREIGN KEY (era_index)
            REFERENCES sub_era (index)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_era_validator_fk_account
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT sub_era_validator_fk_controller_account
        FOREIGN KEY (validator_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_era_validator_idx_era_index
    ON sub_era_validator (era_index);
CREATE INDEX IF NOT EXISTS sub_era_validator_idx_validator_account_id
    ON sub_era_validator (validator_account_id);
CREATE INDEX IF NOT EXISTS sub_era_validator_idx_era_index_validator_account_id
    ON sub_era_validator (era_index, validator_account_id);
CREATE INDEX IF NOT EXISTS sub_era_validator_idx_is_active
    ON sub_era_validator (is_active);
CREATE INDEX IF NOT EXISTS sub_era_validator_idx_active_validator_index
    ON sub_era_validator (active_validator_index);
CREATE INDEX IF NOT EXISTS sub_era_validator_idx_validator_account_id_is_active
    ON sub_era_validator (validator_account_id, is_active);