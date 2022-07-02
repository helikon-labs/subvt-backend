ALTER TABLE sub_session_para_validator ADD COLUMN IF NOT EXISTS active_validator_index bigint NOT NULL;
ALTER TABLE sub_session_para_validator ADD COLUMN IF NOT EXISTS para_validator_group_index bigint NOT NULL;
ALTER TABLE sub_session_para_validator ADD COLUMN IF NOT EXISTS para_validator_index bigint NOT NULL;