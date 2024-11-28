DROP FUNCTION IF EXISTS sub_get_validator_info_batch;

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
		dn_node_record_id INTEGER,
		dn_status text,
		performance TEXT[]
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

    SELECT id, status
    FROM sub_onekv_candidate C
    INTO result_record.dn_node_record_id, result_record.dn_status
    WHERE C.validator_account_id = account_id_param
    ORDER BY id DESC
    LIMIT 1;

    SELECT COALESCE(
        ARRAY_AGG(performance),
        ARRAY[]::TEXT[]
    )
    FROM(
        SELECT (
            era_index::TEXT || ',' ||
            session_index::TEXT || ',' ||
            implicit_attestation_count::TEXT || ',' ||
            explicit_attestation_count::TEXT || ',' ||
            missed_attestation_count::TEXT || ',' ||
            attestations_per_billion::TEXT
        ) AS performance
        FROM sub_session_validator_performance
        WHERE validator_account_id = account_id_param
        AND para_validator_index IS NOT NULL
        ORDER BY id DESC
        LIMIT 10
    ) AS subquery
    INTO result_record.performance;
	
    RETURN result_record;
END
$$ LANGUAGE plpgsql PARALLEL SAFE STABLE;

CREATE OR REPLACE FUNCTION sub_get_validator_info_batch (block_hash_param VARCHAR(66), account_ids_param VARCHAR(66)[], is_active_param boolean[], era_index_param bigint)
RETURNS SETOF sub_validator_info
AS $$

DECLARE
    validator_info sub_validator_info;
    account_id VARCHAR(66);
    i INT = 1;
    is_active BOOLEAN = false;
BEGIN
    FOREACH account_id IN ARRAY account_ids_param
    LOOP
        is_active := is_active_param[i];
        validator_info := sub_get_validator_info(block_hash_param, account_id, is_active, era_index_param);
        RETURN NEXT validator_info;
        i:= i + 1;
    END LOOP;
END
$$ LANGUAGE plpgsql PARALLEL SAFE STABLE;