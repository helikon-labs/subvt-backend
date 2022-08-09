use frankenstein::{Chat, ChatType, Message, MethodResponse};
use subvt_persistence::redis::Redis;
use subvt_types::crypto::AccountId;
use subvt_types::report::BlockSummary;
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

pub async fn set_redis_finalized_block(redis: &Redis) -> anyhow::Result<BlockSummary> {
    let block_summary = BlockSummary {
        number: 13928858,
        hash: "0xAF0FAFE6A27FCEA7327DDC819C5521C4535390AB802EFD5F2B98E277C905A0BE".to_string(),
        timestamp: 1660035252015,
    };
    redis.set_finalized_block_summary(&block_summary).await?;
    Ok(block_summary)
}

pub async fn add_validator_to_redis(redis: &Redis, account_id: &AccountId) -> anyhow::Result<()> {
    let mut validator_details = ValidatorDetails::default();
    validator_details.account.id = *account_id;
    validator_details.account.address = account_id.to_ss58_check();
    let block = set_redis_finalized_block(redis).await?;
    redis
        .add_validator_account_id_to_active_set(block.number, account_id)
        .await?;
    redis
        .set_active_validator_details(block.number, &validator_details)
        .await?;
    Ok(())
}
