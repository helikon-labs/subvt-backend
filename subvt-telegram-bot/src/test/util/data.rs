use crate::test::util::get_random_block_number;
use frankenstein::{Chat, ChatType, Message, MethodResponse};
use subvt_persistence::redis::Redis;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::Account;
use subvt_types::subvt::ValidatorDetails;

pub fn get_telegram_response_message() -> Message {
    let chat = Chat::builder().id(0).type_field(ChatType::Private).build();
    Message::builder().message_id(0).date(0).chat(chat).build()
}

pub fn get_telegram_response() -> MethodResponse<Message> {
    MethodResponse {
        ok: true,
        result: get_telegram_response_message(),
        description: None,
    }
}

pub fn get_dummy_validator_details(account_id: &AccountId) -> ValidatorDetails {
    ValidatorDetails {
        account: Account {
            id: *account_id,
            address: account_id.to_ss58_check(),
            identity: None,
            parent: Box::new(None),
            child_display: None,
            discovered_at: None,
            killed_at: None,
        },
        controller_account_id: Default::default(),
        preferences: Default::default(),
        self_stake: Default::default(),
        reward_destination: Default::default(),
        next_session_keys: "".to_string(),
        queued_session_keys: None,
        is_active: false,
        active_next_session: false,
        nominations: vec![],
        oversubscribed: false,
        active_era_count: 0,
        inactive_era_count: 0,
        slash_count: 0,
        offline_offence_count: 0,
        unclaimed_era_indices: vec![],
        is_para_validator: false,
        para_core_assignment: None,
        return_rate_per_billion: None,
        blocks_authored: None,
        reward_points: None,
        heartbeat_received: None,
        validator_stake: None,
        onekv_candidate_record_id: None,
        onekv_binary_version: None,
        onekv_rank: None,
        onekv_location: None,
        onekv_is_valid: None,
        onekv_online_since: None,
        onekv_offline_since: None,
    }
}

pub async fn add_validator_to_redis(redis: &Redis, account_id: &AccountId) -> anyhow::Result<()> {
    let finalized_block_number = get_random_block_number();
    let validator_details = get_dummy_validator_details(account_id);
    redis
        .set_finalized_block_number(finalized_block_number)
        .await?;
    redis
        .add_validator_account_id_to_active_set(finalized_block_number, account_id)
        .await?;
    redis
        .set_active_validator_details(finalized_block_number, &validator_details)
        .await?;
    Ok(())
}
