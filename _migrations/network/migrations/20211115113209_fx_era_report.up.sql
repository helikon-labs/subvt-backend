DO $$ BEGIN
    IF to_regtype('sub_era_report') IS NULL THEN
        CREATE TYPE sub_era_report AS (
        	start_timestamp bigint,
        	end_timestamp bigint,
        	minimum_stake VARCHAR(128),
        	maximum_stake VARCHAR(128),
        	average_stake VARCHAR(128),
        	median_stake VARCHAR(128),
        	total_validator_reward VARCHAR(128),
        	total_reward_points bigint,
        	total_reward bigint,
        	total_stake VARCHAR(128),
        	active_nominator_count integer,
        	offline_offence_count integer,
        	slashed_amount bigint,
        	chilling_count integer
        );
    END IF;
END $$;

CREATE OR REPLACE FUNCTION sub_get_era_report (era_index_param bigint)
RETURNS sub_era_report
AS $$

DECLARE
    result_record sub_era_report;

BEGIN
	SELECT E.start_timestamp, E.end_timestamp, E.active_nominator_count,
		E.total_stake, E.minimum_stake, E.maximum_stake, E.average_stake, E.median_stake,
		E.total_validator_reward, E.total_reward_points
	FROM sub_era E
	INTO result_record.start_timestamp, result_record.end_timestamp, result_record.active_nominator_count,
		result_record.total_stake, result_record.minimum_stake, result_record.maximum_stake, result_record.average_stake,
		result_record.median_stake, result_record.total_validator_reward, result_record.total_reward_points
	WHERE E.index = era_index_param;
	
	SELECT COALESCE(SUM(ER.amount::bigint), 0)
	FROM sub_event_rewarded ER, sub_extrinsic_payout_stakers EPS
	INTO result_record.total_reward
	WHERE EPS.era_index = era_index_param
	AND EPS.extrinsic_index = ER.extrinsic_index
	AND EPS.block_hash = ER.block_hash
	AND EPS.is_successful = true;
	
	SELECT COUNT(DISTINCT EVO.id)
	FROM sub_event_validator_offline EVO, sub_block B
	INTO result_record.offline_offence_count
	WHERE EVO.block_hash = B.hash
	AND B.era_index = era_index_param;
	
	SELECT COALESCE(SUM(ES.amount::bigint), 0)
	FROM sub_event_slashed ES, sub_block B
	INTO result_record.slashed_amount
	WHERE ES.block_hash = B.hash
	AND B.era_index = era_index_param;
	
	SELECT COUNT(DISTINCT EVC.id)
	FROM sub_event_chilled EVC, sub_block B
	INTO result_record.chilling_count
	WHERE EVC.block_hash = B.hash
	AND B.era_index = era_index_param;

	RETURN result_record;
END
$$ LANGUAGE plpgsql PARALLEL SAFE STABLE;
