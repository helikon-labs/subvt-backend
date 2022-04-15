CREATE TABLE IF NOT EXISTS sub_telegram_chat_validator
(
    id                  SERIAL PRIMARY KEY,
    telegram_chat_id    bigint NOT NULL,
    account_id          VARCHAR(66) NOT NULL,
    address             VARCHAR(64) NOT NULL,
    display             text,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMP WITHOUT TIME ZONE,
    CONSTRAINT sub_telegram_chat_validator_u_chat_validator
        UNIQUE (telegram_chat_id, account_id),
    CONSTRAINT sub_telegram_chat_validator_fk_chat
        FOREIGN KEY (telegram_chat_id)
            REFERENCES sub_telegram_chat (telegram_chat_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_telegram_chat_validator_fk_account
        FOREIGN KEY (account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS sub_telegram_chat_validator_idx_chat_deleted_at
    ON sub_telegram_chat_validator (telegram_chat_id, deleted_at);