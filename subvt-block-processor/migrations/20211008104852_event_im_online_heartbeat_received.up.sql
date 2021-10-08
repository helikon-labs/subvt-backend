CREATE TABLE IF NOT EXISTS event_im_online_heartbeat_received
(
    block_hash      VARCHAR(66)                 NOT NULL,
    extrinsic_index integer,
    account_id      VARCHAR(66)                 NOT NULL,
    era_index       bigint                      NOT NULL,
    epoch_index     bigint                      NOT NULL,
    last_updated    TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT im_online_heartbeat_received_u_block_hash_account_id
        UNIQUE (block_hash, account_id),
    CONSTRAINT im_online_heartbeat_received_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT im_online_heartbeat_received_fk_account
        FOREIGN KEY (account_id)
            REFERENCES account (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT im_online_heartbeat_received_fk_era
        FOREIGN KEY (era_index)
            REFERENCES era (index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT im_online_heartbeat_received_fk_epoch
        FOREIGN KEY (epoch_index)
            REFERENCES epoch (index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);
