use crate::test::util::get_random_block_number;
use frankenstein::{Chat, ChatType, Message, MethodResponse};
use subvt_persistence::redis::Redis;
use subvt_types::crypto::AccountId;
use subvt_types::subvt::ValidatorDetails;

pub fn get_telegram_response_message() -> Message {
    let chat = Chat::builder().id(0).type_field(ChatType::Private).build();
    Message::builder().message_id(0).date(0).chat(chat).build()
}

pub fn get_telegram_message_response() -> MethodResponse<Message> {
    MethodResponse {
        ok: true,
        result: get_telegram_response_message(),
        description: None,
    }
}

pub fn get_telegram_bool_response(result: bool) -> MethodResponse<bool> {
    MethodResponse {
        ok: true,
        result,
        description: None,
    }
}

pub async fn add_validator_to_redis(redis: &Redis, account_id: &AccountId) -> anyhow::Result<()> {
    let finalized_block_number = get_random_block_number();
    let mut validator_details = ValidatorDetails::default();
    validator_details.account.id = *account_id;
    validator_details.account.address = account_id.to_ss58_check();
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
