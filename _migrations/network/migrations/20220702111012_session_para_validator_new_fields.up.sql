ALTER TABLE sub_session_para_validator ADD COLUMN IF NOT EXISTS active_validator_index bigint;
ALTER TABLE sub_session_para_validator ADD COLUMN IF NOT EXISTS para_validator_group_index bigint;
ALTER TABLE sub_session_para_validator ADD COLUMN IF NOT EXISTS para_validator_index bigint;

CREATE INDEX IF NOT EXISTS sub_session_para_validator_idx_session_validator_para_validator
    ON sub_session_para_validator (session_index, validator_account_id, para_validator_index);