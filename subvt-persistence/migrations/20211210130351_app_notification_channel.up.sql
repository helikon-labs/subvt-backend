CREATE TABLE IF NOT EXISTS app_notification_channel
(
    name        VARCHAR(16) PRIMARY KEY,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

INSERT INTO app_notification_channel(name) VALUES('apns');
INSERT INTO app_notification_channel(name) VALUES('fcm');
INSERT INTO app_notification_channel(name) VALUES('telegram');
INSERT INTO app_notification_channel(name) VALUES('email');
INSERT INTO app_notification_channel(name) VALUES('gsm');