CREATE MATERIALIZED VIEW IF NOT EXISTS sub_mat_view_validator_reward
AS
    SELECT ER.id, ER.rewardee_account_id as validator_account_id, ER.amount, ER.block_hash, B.timestamp
    FROM sub_event_rewarded ER
    INNER JOIN sub_block B
    ON ER.block_hash = B.hash
    WHERE EXISTS (
        SELECT id FROM sub_era_validator EV
        WHERE EV.validator_account_id = ER.rewardee_account_id
    )
WITH DATA;

CREATE UNIQUE INDEX IF NOT EXISTS sub_mat_view_validator_reward_u_id
    ON sub_mat_view_validator_reward (id);
CREATE INDEX IF NOT EXISTS sub_mat_view_validator_reward_idx_validator_account_id
    ON sub_mat_view_validator_reward (validator_account_id);
CREATE INDEX IF NOT EXISTS sub_mat_view_validator_reward_idx_timestamp
    ON sub_mat_view_validator_reward (timestamp);