CREATE TABLE IF NOT EXISTS app_user_validator
(
    id                      SERIAL PRIMARY KEY,
    user_id                 integer NOT NULL,
    network_id              integer NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMP WITHOUT TIME ZONE,
    CONSTRAINT app_user_validator_u_user_network_validator
        UNIQUE (user_id, network_id, validator_account_id),
    CONSTRAINT app_user_validator_fk_user
        FOREIGN KEY (user_id)
            REFERENCES app_user (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_user_validator_fk_network
        FOREIGN KEY (network_id)
            REFERENCES app_network (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX app_user_validator_idx_user_id
    ON app_user_validator (user_id);

CREATE INDEX app_user_validator_idx_user_id_network_id
    ON app_user_validator (user_id, network_id);

CREATE INDEX app_user_validator_idx_validator_account_id
    ON app_user_validator (validator_account_id);

CREATE INDEX app_user_validator_idx_search_1
    ON app_user_validator (network_id, validator_account_id, deleted_at);

CREATE INDEX app_user_validator_idx_search_2
    ON app_user_validator (id, network_id, validator_account_id, deleted_at);