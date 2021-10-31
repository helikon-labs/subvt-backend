CREATE TYPE validator_info AS (
    discovered_at bigint,
	killed_at bigint,
	slash_count bigint,
	offline_offence_count bigint,
	active_era_count bigint,
	inactive_era_count bigint,
	total_reward_points bigint,
	unclaimed_eras text
);

CREATE OR REPLACE FUNCTION get_validator_info (account_id VARCHAR(66))
RETURNS validator_info
AS $$

DECLARE
	result_record validator_info;

BEGIN
	SELECT COUNT(DISTINCT id)
	INTO result_record.slash_count
	FROM event_slashed
	WHERE validator_account_id = account_id;
	
	SELECT COUNT(DISTINCT block_hash)
	INTO result_record.offline_offence_count
	FROM event_validator_offline
	WHERE validator_account_id = account_id;
	
	SELECT COUNT(DISTINCT era_index), COALESCE(SUM(reward_points))
	INTO result_record.active_era_count, result_record.total_reward_points
	FROM era_validator
	WHERE validator_account_id = account_id
	AND is_active = true;
	
	SELECT COUNT(DISTINCT era_index)
	INTO result_record.inactive_era_count
	FROM era_validator
	WHERE validator_account_id = account_id
	AND is_active = false;
	
	SELECT STRING_AGG(EV.era_index::character varying, ',')
	INTO result_record.unclaimed_eras
	FROM era_validator EV
	WHERE EV.validator_account_id = account_id
	AND EV.is_active = true
	AND NOT EXISTS(
		SELECT 1
		FROM extrinsic_payout_stakers EPS
		WHERE EPS.validator_account_id = account_id
		AND EPS.era_index = EV.era_index
		AND is_successful = true
	);

    SELECT block.timestamp
	INTO result_record.discovered_at
	FROM block, account
	WHERE account.discovered_at_block_hash = block.hash
	AND account.id = account_id;
	
	SELECT block.timestamp
	INTO result_record.killed_at
	FROM block, account
	WHERE account.killed_at_block_hash = block.hash
	AND account.id = account_id;
	
	RETURN result_record;
END
$$ LANGUAGE plpgsql PARALLEL SAFE STABLE;
