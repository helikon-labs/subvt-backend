CREATE TABLE IF NOT EXISTS sub_onekv_nominator
(
    id                      SERIAL PRIMARY KEY,
    onekv_id                VARCHAR(128) NOT NULL,
    account_id              VARCHAR(66) NOT NULL,
    stash_account_id        VARCHAR(66) NOT NULL,
    proxy_account_id        VARCHAR(66) NOT NULL,
    bonded_amount           VARCHAR(128) NOT NULL,
    proxy_delay             INTEGER NOT NULL,
    last_nomination_at      bigint NOT NULL,
    nominator_created_at    bigint NOT NULL,
    average_stake           double precision NOT NULL,
    new_bonded_amount       double precision NOT NULL,
    reward_destination      VARCHAR(128) NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_onekv_nominator_fk_account_id
        FOREIGN KEY (account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT sub_onekv_nominator_fk_stash_account_id
        FOREIGN KEY (stash_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT sub_onekv_nominator_fk_proxy_account_id
        FOREIGN KEY (proxy_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);