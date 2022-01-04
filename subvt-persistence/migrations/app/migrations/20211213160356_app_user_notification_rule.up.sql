CREATE TYPE app_notification_period_type AS ENUM ('immediate', 'hour', 'day', 'epoch', 'era');

CREATE TABLE IF NOT EXISTS app_user_notification_rule
(
    id                      SERIAL PRIMARY KEY,
    user_id                 integer NOT NULL,
    notification_type_code  VARCHAR(256) NOT NULL,
    name                    text,
    network_id              integer,
    is_for_all_validators   boolean NOT NULL,
    period_type             app_notification_period_type NOT NULL default 'immediate',
    period                  integer NOT NULL default 0,
    notes                   text,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMP WITHOUT TIME ZONE,
    CONSTRAINT app_user_notification_rule_fk_user
        FOREIGN KEY (user_id)
            REFERENCES app_user (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_user_notification_rule_fk_notification_type
        FOREIGN KEY (notification_type_code)
            REFERENCES app_notification_type (code)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_user_notification_rule_fk_network
        FOREIGN KEY (network_id)
            REFERENCES app_network (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX app_user_notification_rule_idx_user_id
    ON app_user_notification_rule (user_id);

CREATE INDEX app_user_notification_rule_idx_search
    ON app_user_notification_rule (
        notification_type_code,
        deleted_at,
        network_id,
        is_for_all_validators
    );

CREATE TABLE IF NOT EXISTS app_user_notification_rule_validator
(
    user_notification_rule_id   integer NOT NULL,
    user_validator_id           integer NOT NULL,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT app_user_notification_rule_validator_pk
        PRIMARY KEY (user_notification_rule_id, user_validator_id),
    CONSTRAINT app_user_notification_rule_validator_fk_rule
        FOREIGN KEY (user_notification_rule_id)
            REFERENCES app_user_notification_rule (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_user_notification_rule_validator_fk_user_validator
        FOREIGN KEY (user_validator_id)
            REFERENCES app_user_validator (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX app_user_notification_rule_validator_idx_rule_id
    ON app_user_notification_rule_validator (user_notification_rule_id);

CREATE INDEX app_user_notification_rule_validator_idx_user_validator_id
    ON app_user_notification_rule_validator (user_validator_id);

CREATE INDEX app_user_notification_rule_validator_idx_search
    ON app_user_notification_rule_validator (user_notification_rule_id, user_validator_id);

CREATE TABLE IF NOT EXISTS app_user_notification_rule_channel
(
    user_notification_rule_id       integer NOT NULL,
    user_notification_channel_id    integer NOT NULL,
    created_at                      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT app_user_notification_rule_channel_pk
        PRIMARY KEY (user_notification_rule_id, user_notification_channel_id),
    CONSTRAINT app_user_notification_rule_channel_fk_rule
        FOREIGN KEY (user_notification_rule_id)
            REFERENCES app_user_notification_rule (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_user_notification_rule_channel_fk_channel
        FOREIGN KEY (user_notification_channel_id)
            REFERENCES app_user_notification_channel (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX app_user_notification_rule_channel_idx_rule_id
    ON app_user_notification_rule_channel (user_notification_rule_id);

CREATE INDEX app_user_notification_rule_channel_idx_channel_id
    ON app_user_notification_rule_channel (user_notification_channel_id);

CREATE TABLE IF NOT EXISTS app_user_notification_rule_param
(
    user_notification_rule_id   integer NOT NULL,
    notification_param_type_id  integer NOT NULL,
    "value"                     VARCHAR(128),
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT app_user_notification_rule_param_pk
        PRIMARY KEY (user_notification_rule_id, notification_param_type_id),
    CONSTRAINT app_user_notification_rule_channel_fk_rule
        FOREIGN KEY (user_notification_rule_id)
            REFERENCES app_user_notification_rule (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_user_notification_rule_param_fk_param_type
        FOREIGN KEY (notification_param_type_id)
            REFERENCES app_notification_param_type (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX app_user_notification_rule_param_idx_rule_id
    ON app_user_notification_rule_channel (user_notification_rule_id);

CREATE INDEX app_user_notification_rule_param_idx_param_id
    ON app_user_notification_rule_param (notification_param_type_id);