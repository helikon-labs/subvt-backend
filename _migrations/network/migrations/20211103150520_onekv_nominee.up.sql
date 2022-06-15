CREATE TABLE IF NOT EXISTS sub_onekv_nominee
(
    id                  SERIAL PRIMARY KEY,
    onekv_nominator_id  INTEGER NOT NULL,
    stash_account_id    VARCHAR(128) NOT NULL,
    name                text NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_onekv_nominee_fk_nominator_id
        FOREIGN KEY (onekv_nominator_id)
            REFERENCES sub_onekv_nominator (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    CONSTRAINT sub_onekv_nominee_fk_stash_account_id
        FOREIGN KEY (stash_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);