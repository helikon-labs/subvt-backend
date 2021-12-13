CREATE TABLE IF NOT EXISTS app_notification_channel
(
    code        VARCHAR(16) PRIMARY KEY,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

INSERT INTO app_notification_channel(code) VALUES('apns');
INSERT INTO app_notification_channel(code) VALUES('fcm');
INSERT INTO app_notification_channel(code) VALUES('telegram');
INSERT INTO app_notification_channel(code) VALUES('email');
INSERT INTO app_notification_channel(code) VALUES('gsm');