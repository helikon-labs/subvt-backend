CREATE TABLE IF NOT EXISTS app_user_notification_channel
(
    id                          SERIAL PRIMARY KEY,
    user_id                     INTEGER NOT NULL,
    notification_channel_code   VARCHAR(16) NOT NULL,
    target                      VARCHAR(1024) NOT NULL,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    deleted_at                  TIMESTAMP WITHOUT TIME ZONE,
    CONSTRAINT app_user_notification_channel_fk_user
        FOREIGN KEY (user_id)
            REFERENCES app_user (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT app_user_notification_channel_fk_channel
        FOREIGN KEY (notification_channel_code)
            REFERENCES app_notification_channel (code)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS app_user_notification_channel_u_user_channel_target
    ON app_user_notification_channel (user_id, notification_channel_code, target)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS app_user_notification_channel_u_user_channel_target_deleted
    ON app_user_notification_channel (user_id, notification_channel_code, target, deleted_at)
    WHERE deleted_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS app_user_notification_channel_idx_user_id
    ON app_user_notification_channel (user_id);
CREATE INDEX IF NOT EXISTS app_user_notification_channel_idx_user_id_channel
    ON app_user_notification_channel (user_id, notification_channel_code);