CREATE TABLE IF NOT EXISTS app_notification
(
    id                              SERIAL PRIMARY KEY,
    user_id                         integer NOT NULL,
    user_notification_rule_id       integer NOT NULL,
    network_id                      integer NOT NULL,
    period_type                     app_notification_period_type NOT NULL,
    period                          integer NOT NULL,
    validator_account_id            VARCHAR(66) NOT NULL,
    notification_type_code          VARCHAR(256) NOT NULL,
    param_type_id                   integer,
    param_value                     VARCHAR(128),
    block_hash                      VARCHAR(66),
    block_number                    bigint,
    block_timestamp                 bigint,
    extrinsic_index                 integer,
    event_index                     integer,
    user_notification_channel_id    integer NOT NULL,
    notification_channel_code       VARCHAR(16) NOT NULL,
    notification_target             VARCHAR(1024) NOT NULL,
    data_json                       text,
    log                             text,
    created_at                      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    sent_at                         TIMESTAMP WITHOUT TIME ZONE,
    delivered_at                    TIMESTAMP WITHOUT TIME ZONE,
    read_at                         TIMESTAMP WITHOUT TIME ZONE,
    CONSTRAINT app_notification_fk_user
        FOREIGN KEY (user_id)
            REFERENCES app_user (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_notification_fk_user_notification_rule
        FOREIGN KEY (user_notification_rule_id)
            REFERENCES app_user_notification_rule (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_notification_fk_network
        FOREIGN KEY (network_id)
            REFERENCES app_network (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_notification_fk_notification_type
        FOREIGN KEY (notification_type_code)
            REFERENCES app_notification_type (code)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_notification_fk_param_type
        FOREIGN KEY (param_type_id)
            REFERENCES app_notification_param_type (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_notification_fk_user_notification_channel
        FOREIGN KEY (user_notification_channel_id)
            REFERENCES app_user_notification_channel (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_notification_fk_notification_channel
        FOREIGN KEY (notification_channel_code)
            REFERENCES app_notification_channel (code)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);