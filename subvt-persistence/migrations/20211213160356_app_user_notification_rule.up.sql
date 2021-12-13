CREATE TABLE IF NOT EXISTS app_user_notification_rule
(
    id                      SERIAL PRIMARY KEY,
    user_id                 SERIAL NOT NULL,
    notification_type_code  VARCHAR(256) NOT NULL,
    network_id              SERIAL,
    is_for_all_validators   boolean NOT NULL,
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
            ON UPDATE CASCADE
);

CREATE INDEX app_user_notification_rule_idx_user_id
    ON app_user_notification_rule (user_id);

CREATE TABLE IF NOT EXISTS app_user_notification_rule_validator
(
    user_notification_rule_id   SERIAL NOT NULL,
    user_validator_id           SERIAL NOT NULL,
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
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX app_user_notification_rule_validator_idx_rule_id
    ON app_user_notification_rule_validator (user_notification_rule_id);

CREATE INDEX app_user_notification_rule_validator_idx_user_validator_id
    ON app_user_notification_rule_validator (user_validator_id);

CREATE TABLE IF NOT EXISTS app_user_notification_rule_channel
(
    user_notification_rule_id       SERIAL NOT NULL,
    user_notification_channel_id    SERIAL NOT NULL,
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
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX app_user_notification_rule_channel_idx_rule_id
    ON app_user_notification_rule_channel (user_notification_rule_id);

CREATE INDEX app_user_notification_rule_channel_idx_channel_id
    ON app_user_notification_rule_channel (user_notification_channel_id);

CREATE TABLE IF NOT EXISTS app_user_notification_rule_param
(
    user_notification_rule_id   SERIAL NOT NULL,
    notification_param_type_id  SERIAL NOT NULL,
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
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX app_user_notification_rule_param_idx_rule_id
    ON app_user_notification_rule_channel (user_notification_rule_id);

CREATE INDEX app_user_notification_rule_param_idx_param_id
    ON app_user_notification_rule_param (notification_param_type_id);