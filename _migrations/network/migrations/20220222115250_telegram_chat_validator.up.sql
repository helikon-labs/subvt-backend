CREATE TABLE IF NOT EXISTS sub_telegram_chat_validator
(
    id                      SERIAL PRIMARY KEY,
    telegram_chat_id        bigint NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMP WITHOUT TIME ZONE
);

ALTER TABLE sub_telegram_chat_validator
    ADD CONSTRAINT sub_telegram_chat_validator_u_chat_validator
    UNIQUE (telegram_chat_id, validator_account_id);

ALTER TABLE sub_telegram_chat_validator
    ADD CONSTRAINT sub_telegram_chat_validator_fk_chat
    FOREIGN KEY (telegram_chat_id)
        REFERENCES sub_telegram_chat (telegram_chat_id)
        ON DELETE CASCADE
        ON UPDATE CASCADE;
ALTER TABLE sub_telegram_chat_validator
    ADD CONSTRAINT sub_telegram_chat_validator_fk_account
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;