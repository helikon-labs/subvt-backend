CREATE INDEX IF NOT EXISTS sub_session_para_validator_idx_validator_account_id_id
    ON sub_session_para_validator (validator_account_id, id DESC);