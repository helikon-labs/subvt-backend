CREATE TABLE IF NOT EXISTS sub_epoch
(
    index               bigint PRIMARY KEY,
    era_index           bigint NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_epoch
    ADD CONSTRAINT sub_epoch_fk_era
    FOREIGN KEY (era_index)
        REFERENCES sub_era (index)
        ON DELETE CASCADE
        ON UPDATE CASCADE;

CREATE INDEX sub_epoch_idx_era
    ON sub_epoch (era_index);