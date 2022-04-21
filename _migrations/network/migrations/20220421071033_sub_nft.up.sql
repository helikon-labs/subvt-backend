CREATE TABLE IF NOT EXISTS sub_nft
(
    id                  VARCHAR(128) NOT NULL,
    chain               VARCHAR(128) NOT NULL,
    owner_account_id    VARCHAR(66) NOT NULL,
    content_type        text,
    name                text,
    description         text,
    url                 text,
    image_url           text,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    PRIMARY KEY (id, chain, owner_account_id),
    CONSTRAINT sub_nft_fk_account
        FOREIGN KEY (owner_account_id)
            REFERENCES sub_account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX sub_nft_idx_owner_account_id
    ON sub_nft (owner_account_id);