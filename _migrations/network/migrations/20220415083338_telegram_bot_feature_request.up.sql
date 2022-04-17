CREATE TABLE IF NOT EXISTS telegram_bot_feature_request
(
    telegram_chat_id    bigint NOT NULL,
    content             text NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT telegram_bot_feature_request_fk_chat
        FOREIGN KEY (telegram_chat_id)
            REFERENCES sub_telegram_chat (telegram_chat_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);