CREATE TABLE IF NOT EXISTS app_notification_channel
(
    code        VARCHAR(16) PRIMARY KEY,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

INSERT INTO app_notification_channel(code) VALUES('apns') ON CONFLICT(code) DO NOTHING;
INSERT INTO app_notification_channel(code) VALUES('fcm') ON CONFLICT(code) DO NOTHING;
INSERT INTO app_notification_channel(code) VALUES('telegram') ON CONFLICT(code) DO NOTHING;
INSERT INTO app_notification_channel(code) VALUES('email') ON CONFLICT(code) DO NOTHING;
INSERT INTO app_notification_channel(code) VALUES('gsm') ON CONFLICT(code) DO NOTHING;
INSERT INTO app_notification_channel(code) VALUES('sms') ON CONFLICT(code) DO NOTHING;