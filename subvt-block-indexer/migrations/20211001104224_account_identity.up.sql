CREATE TABLE IF NOT EXISTS account_identity
(
    account_id   VARCHAR(64) PRIMARY KEY,
    display      VARCHAR(2048),
    email        VARCHAR(2048),
    riot         VARCHAR(2048),
    twitter      VARCHAR(2048),
    confirmed    BOOLEAN                     NOT NULL DEFAULT FALSE,
    last_updated TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT fk_account
        FOREIGN KEY (account_id)
            REFERENCES account (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
