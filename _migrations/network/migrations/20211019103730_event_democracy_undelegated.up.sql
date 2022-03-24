CREATE TABLE IF NOT EXISTS sub_event_democracy_undelegated
(
    id              SERIAL PRIMARY KEY,
    block_hash      VARCHAR(66) NOT NULL,
    extrinsic_index integer,
    event_index     integer NOT NULL,
    account_id      VARCHAR(66) NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_event_democracy_undelegated
    ADD CONSTRAINT sub_event_democracy_undelegated_fk_block
    FOREIGN KEY (block_hash)
        REFERENCES sub_block (hash)
        ON DELETE CASCADE
        ON UPDATE CASCADE;

ALTER TABLE sub_event_democracy_undelegated
    ADD CONSTRAINT sub_event_democracy_undelegated_fk_account
    FOREIGN KEY (account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;

CREATE INDEX sub_event_democracy_undelegated_idx_block_hash
    ON sub_event_democracy_undelegated (block_hash);
CREATE INDEX sub_event_democracy_undelegated_idx_account
    ON sub_event_democracy_undelegated (account_id);