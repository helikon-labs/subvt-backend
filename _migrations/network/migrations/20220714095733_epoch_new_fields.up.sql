ALTER TABLE sub_epoch ADD COLUMN IF NOT EXISTS start_block_number bigint NOT NULL DEFAULT 0;
ALTER TABLE sub_epoch ADD COLUMN IF NOT EXISTS start_timestamp bigint NOT NULL DEFAULT 0;
ALTER TABLE sub_epoch ADD COLUMN IF NOT EXISTS end_timestamp bigint NOT NULL DEFAULT 0;