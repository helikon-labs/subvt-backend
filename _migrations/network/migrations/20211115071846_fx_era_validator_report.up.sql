DO $$ BEGIN
    IF to_regtype('sub_era_validator_report') IS NULL THEN
        CREATE TYPE sub_era_validator_report AS (
        	era_start_timestamp bigint,
        	era_end_timestamp bigint,
            is_active boolean,
        	commission_per_billion bigint,
        	self_stake VARCHAR(128),
        	total_stake VARCHAR(128),
        	block_count INTEGER,
        	reward_points INTEGER,
        	self_reward bigint,
        	staker_reward bigint,
        	offline_offence_count INTEGER,
        	slashed_amount bigint,
        	chilling_count INTEGER
        );
    END IF;
END $$;

CREATE OR REPLACE FUNCTION sub_get_era_validator_report (era_index_param bigint, account_id_param VARCHAR(66))
RETURNS sub_era_validator_report
AS $$

DECLARE
    result_record sub_era_validator_report;

BEGIN
	SELECT E.start_timestamp, E.end_timestamp
	FROM sub_era E
	INTO result_record.era_start_timestamp, result_record.era_end_timestamp
	WHERE E.index = era_index_param;

	SELECT is_active, commission_per_billion, self_stake, total_stake, reward_points
	FROM sub_era_validator
	INTO result_record.is_active, result_record.commission_per_billion,
	    result_record.self_stake, result_record.total_stake, result_record.reward_points
	WHERE validator_account_id = account_id_param
	AND era_index = era_index_param;

	SELECT COUNT(DISTINCT B.number)
	FROM sub_block B
	INTO result_record.block_count
	WHERE B.author_account_id = account_id_param
	AND B.era_index = era_index_param;

	SELECT COALESCE(SUM(EV.amount::bigint), 0)
    FROM sub_event_rewarded EV
    INTO result_record.self_reward
    INNER JOIN sub_extrinsic_payout_stakers EX
    	ON EV.block_hash = EX.block_hash
    	AND EV.extrinsic_index = EX.extrinsic_index
    	AND COALESCE(EV.nesting_index, '') = COALESCE(EX.nesting_index, '')
    	AND EX.validator_account_id = EV.rewardee_account_id
    INNER JOIN sub_era E
    	ON E.index = EX.era_index
    	AND E.index = era_index_param
    WHERE EV.rewardee_account_id = account_id_param;

    SELECT COALESCE(SUM(EV.amount::bigint), 0)
    FROM sub_event_rewarded EV
    INTO result_record.staker_reward
    INNER JOIN sub_extrinsic_payout_stakers EX
        ON EV.block_hash = EX.block_hash
        AND EV.extrinsic_index = EX.extrinsic_index
        AND COALESCE(EV.nesting_index, '') = COALESCE(EX.nesting_index, '')
        AND EX.validator_account_id != EV.rewardee_account_id
    INNER JOIN sub_era E
        ON E.index = EX.era_index
        AND E.index = era_index_param
    WHERE EX.validator_account_id = account_id_param;

	SELECT COUNT(DISTINCT EVO.id)
	FROM sub_event_validator_offline EVO, sub_block B
	INTO result_record.offline_offence_count
	WHERE EVO.validator_account_id = account_id_param
	AND EVO.block_hash = B.hash
	AND B.era_index = era_index_param;

	SELECT COALESCE(SUM(ES.amount::bigint), 0)
	FROM sub_event_slashed ES, sub_block B
	INTO result_record.slashed_amount
	WHERE ES.validator_account_id = account_id_param
	AND ES.block_hash = B.hash
	AND B.era_index = era_index_param;

	SELECT COUNT(DISTINCT EC.id)
	FROM sub_event_chilled EC, sub_block B
	INTO result_record.chilling_count
	WHERE EC.stash_account_id = account_id_param
	AND EC.stash_account_id = B.hash
	AND B.era_index = era_index_param;

	RETURN result_record;
END
$$ LANGUAGE plpgsql PARALLEL SAFE STABLE;