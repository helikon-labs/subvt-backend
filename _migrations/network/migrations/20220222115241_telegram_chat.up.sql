CREATE TABLE IF NOT EXISTS sub_telegram_chat
(
    telegram_chat_id    bigint PRIMARY KEY,
    app_user_id         INTEGER NOT NULL,
    settings_message_id INTEGER,
    state               VARCHAR(128) NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMP WITHOUT TIME ZONE
);