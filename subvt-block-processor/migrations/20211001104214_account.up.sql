CREATE TABLE IF NOT EXISTS account
(
    id            VARCHAR(64) PRIMARY KEY,
    parent_id     VARCHAR(64),
    discovered_at bigint,
    last_updated  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT fk_parent_account
        FOREIGN KEY (parent_id)
            REFERENCES account (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);
