DO $$ BEGIN
	DROP FUNCTION IF EXISTS sub_get_validator_info;
	DROP TYPE IF EXISTS sub_validator_info;
	CREATE TYPE sub_validator_info AS (
		discovered_at bigint,
		slash_count bigint,
		offline_offence_count bigint,
		active_era_count bigint,
		inactive_era_count bigint,
		unclaimed_eras text,
		blocks_authored bigint,
		reward_points bigint,
		heartbeat_received boolean,
		onekv_candidate_record_id INTEGER,
		onekv_binary_version text,
		onekv_rank bigint,
		onekv_location text,
		onekv_is_valid boolean,
		onekv_online_since bigint,
		onekv_offline_since bigint
	);
END $$;

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
	
    SELECT COUNT(DISTINCT era_index)
    INTO result_record.active_era_count
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
    INNER JOIN sub_era E
        ON EV.era_index = E.index
    WHERE EV.validator_account_id = account_id_param
    AND EV.is_active = true
    AND EV.reward_points > 0
    AND E.end_timestamp < (EXTRACT(epoch FROM now() AT time zone 'UTC')::bigint * 1000)
    AND E.start_timestamp > (EXTRACT(epoch FROM now() AT time zone 'UTC')::bigint * 1000 - (90::bigint * 24 * 60 * 60 * 1000))
    AND NOT EXISTS(
        SELECT 1
        FROM sub_event_payout_started EPS
        WHERE EPS.validator_account_id = account_id_param
        AND EPS.era_index = EV.era_index
    );

    SELECT A.discovered_at
    INTO result_record.discovered_at
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
            FROM sub_extrinsic_heartbeat E
            INNER JOIN sub_block B
                ON E.session_index = B.epoch_index
            WHERE E.validator_account_id = account_id_param
            AND B.hash = block_hash_param
            AND E.is_successful = true
        ) INTO result_record.heartbeat_received;
    end if;

    SELECT id, rank, location, is_valid, version, online_since, offline_since
    FROM sub_onekv_candidate C
    INTO result_record.onekv_candidate_record_id, result_record.onekv_rank, result_record.onekv_location, result_record.onekv_is_valid, result_record.onekv_binary_version, result_record.onekv_online_since, result_record.onekv_offline_since
    WHERE C.validator_account_id = account_id_param
    ORDER BY id DESC
    LIMIT 1;
	
    RETURN result_record;
END
$$ LANGUAGE plpgsql PARALLEL SAFE STABLE;
