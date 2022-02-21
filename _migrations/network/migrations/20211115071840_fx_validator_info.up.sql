CREATE TYPE sub_validator_info AS (
    discovered_at bigint,
    killed_at bigint,
    slash_count bigint,
    offline_offence_count bigint,
    active_era_count bigint,
    inactive_era_count bigint,
    total_reward_points bigint,
    unclaimed_eras text,
    blocks_authored bigint,
    reward_points bigint,
    heartbeat_received boolean,
    onekv_candidate_record_id integer,
    onekv_rank bigint,
    onekv_is_valid boolean
);

CREATE OR REPLACE FUNCTION sub_get_validator_info (block_hash_param VARCHAR(66), account_id_param VARCHAR(66), is_active_param boolean, era_index_param bigint)
RETURNS sub_validator_info
AS $$

DECLARE
    result_record sub_validator_info;

BEGIN
    SELECT COUNT(DISTINCT id)
    INTO result_record.slash_count
    FROM sub_event_slashed
    WHERE validator_account_id = account_id_param;
	
    SELECT COUNT(DISTINCT block_hash)
    INTO result_record.offline_offence_count
    FROM sub_event_validator_offline
    WHERE validator_account_id = account_id_param;
	
    SELECT COUNT(DISTINCT era_index), COALESCE(SUM(reward_points), 0)
    INTO result_record.active_era_count, result_record.total_reward_points
    FROM sub_era_validator
    WHERE validator_account_id = account_id_param
    AND is_active = true;
	
    SELECT COUNT(DISTINCT era_index)
    INTO result_record.inactive_era_count
    FROM sub_era_validator
    WHERE validator_account_id = account_id_param
    AND is_active = false;
	
    SELECT STRING_AGG(EV.era_index::character varying, ',')
    INTO result_record.unclaimed_eras
    FROM sub_era_validator EV
    WHERE EV.validator_account_id = account_id_param
    AND EV.is_active = true
    AND NOT EXISTS(
        SELECT 1
        FROM sub_extrinsic_payout_stakers EPS
        WHERE EPS.validator_account_id = account_id_param
        AND EPS.era_index = EV.era_index
        AND is_successful = true
    );

    SELECT A.discovered_at
    INTO result_record.discovered_at
    FROM sub_account A
    WHERE A.id = account_id_param;
	
    SELECT A.killed_at
    INTO result_record.killed_at
    FROM sub_account A
    WHERE A.id = account_id_param;

    if is_active_param then
        SELECT COUNT(DISTINCT number)
        FROM sub_block
        INTO result_record.blocks_authored
        WHERE era_index = era_index_param
        AND author_account_id = account_id_param;

        SELECT COALESCE(reward_points, 0)
        FROM sub_era_validator
        INTO result_record.reward_points
        WHERE era_index = era_index_param
        AND validator_account_id = account_id_param;

        SELECT EXISTS(
            SELECT E.id
            FROM sub_extrinsic_heartbeat E, sub_block B
            WHERE E.validator_account_id = account_id_param
            AND B.hash = block_hash_param
            AND E.session_index = B.epoch_index
            AND E.is_successful = true
        ) INTO result_record.heartbeat_received;
    end if;

    SELECT id, rank, is_valid
    FROM sub_onekv_candidate C
    INTO result_record.onekv_candidate_record_id, result_record.onekv_rank, result_record.onekv_is_valid
    WHERE C.validator_account_id = account_id_param
    ORDER BY id DESC
    LIMIT 1;
	
    RETURN result_record;
END
$$ LANGUAGE plpgsql PARALLEL SAFE STABLE;
