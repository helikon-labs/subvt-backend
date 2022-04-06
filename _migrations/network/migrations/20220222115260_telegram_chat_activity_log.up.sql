CREATE TABLE IF NOT EXISTS sub_telegram_chat_activity_log
(
    id                  SERIAL PRIMARY KEY,
    telegram_chat_id    bigint NOT NULL,
    command             text,
    query               text,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_telegram_chat_activity_log
    ADD CONSTRAINT sub_telegram_chat_activity_log_fk_chat
    FOREIGN KEY (telegram_chat_id)
        REFERENCES sub_telegram_chat (telegram_chat_id)
        ON DELETE CASCADE
        ON UPDATE CASCADE;

CREATE INDEX sub_telegram_chat_activity_log_idx_chat_id
    ON sub_telegram_chat_activity_log (telegram_chat_id);
CREATE INDEX sub_telegram_chat_activity_log_idx_command
    ON sub_telegram_chat_activity_log (command);
CREATE INDEX sub_telegram_chat_activity_log_idx_query
    ON sub_telegram_chat_activity_log (query);