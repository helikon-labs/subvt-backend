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