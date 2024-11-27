CREATE TABLE IF NOT EXISTS sub_session_validator_performance
(
    id                          SERIAL PRIMARY KEY,
    validator_account_id        VARCHAR(66) NOT NULL,
    era_index                   BIGINT NOT NULL,
    session_index               BIGINT NOT NULL,
    active_validator_index      BIGINT NOT NULL,
    authored_block_count        INT NOT NULL DEFAULT 0,
    para_validator_group_index  BIGINT,
    para_validator_index        BIGINT,
    implicit_attestation_count  INT,
    explicit_attestation_count  INT,
    missed_attestation_count    INT,
    attestations_per_billion    INT,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS sub_session_validator_performance_idx_validator_account_id
    ON sub_session_validator_performance (validator_account_id);
    CREATE INDEX IF NOT EXISTS sub_session_validator_performance_idx_session_index
        ON sub_session_validator_performance (session_index);
CREATE INDEX IF NOT EXISTS sub_session_validator_performance_idx_validator_account_id_session_index_desc
    ON sub_session_validator_performance (validator_account_id, session_index DESC);
CREATE INDEX IF NOT EXISTS sub_session_validator_performance_idx_validator_account_id_active_validator_index_session_index_desc
    ON sub_session_validator_performance (validator_account_id, active_validator_index, session_index DESC);
CREATE INDEX IF NOT EXISTS sub_session_validator_performance_idx_validator_account_id_para_validator_index_session_index_desc
    ON sub_session_validator_performance (validator_account_id, para_validator_index, session_index DESC);