use crate::messenger::MockMessenger;
use crate::query::QueryType;
use crate::test::util::data::{add_validator_to_redis, get_telegram_message_response};
use crate::test::util::{get_random_account_id, get_random_chat_id, new_test_bot};
use crate::MessageType;
use rand::Rng;

/// Tests the case when the user calls the /nominationdetails (alias /nd) command
/// before adding any validators to the chat.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_nomination_details_no_validator() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::NoValidatorsOnChat)
        })
        .times(2)
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    bot.save_or_restore_chat(chat_id).await.unwrap();
    bot.process_command(chat_id, "/nominationdetails", &[])
        .await
        .unwrap();
    bot.process_command(chat_id, "/nd", &[]).await.unwrap();
}

/// Tests /nominationdetails command for a chat with a single validator.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_nomination_details_single_validator() {
    let chat_id = get_random_chat_id();
    let account_id = get_random_account_id();
    let address = account_id.to_ss58_check();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(
            move |_, _, _, message_type: &Box<MessageType>| match &**message_type {
                MessageType::NominationDetails {
                    validator_details, ..
                } => validator_details.account.address == address,
                _ => false,
            },
        )
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    bot.save_or_restore_chat(chat_id).await.unwrap();
    add_validator_to_redis(&bot.redis, &account_id)
        .await
        .unwrap();
    bot.network_postgres
        .add_validator_to_chat(chat_id, &account_id, &account_id.to_ss58_check(), &None)
        .await
        .unwrap();
    bot.process_command(chat_id, "/nominationdetails", &[])
        .await
        .unwrap();
}

/// Tests /nominationdetails for a single validator that doesn't exist in the Redis database.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_nomination_details_single_non_existent_validator() {
    let chat_id = get_random_chat_id();
    let account_id = get_random_account_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(&**message_type, MessageType::ValidatorNotFound { .. })
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    bot.save_or_restore_chat(chat_id).await.unwrap();
    bot.network_postgres
        .add_validator_to_chat(chat_id, &account_id, &account_id.to_ss58_check(), &None)
        .await
        .unwrap();
    bot.process_command(chat_id, "/nominationdetails", &[])
        .await
        .unwrap();
}

/// Tests /nominationdetails where the user has multiple validators added to the chat - should
/// respond with a list of validators for the user to pick one from.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_nomination_details_multiple_validators() {
    let mut rng = rand::rng();
    let validator_count = rng.random_range(3..15);
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(
            move |_, _, _, message_type: &Box<MessageType>| match &**message_type {
                MessageType::ValidatorList {
                    validators,
                    query_type,
                } => {
                    validators.len() == validator_count
                        && matches!(query_type, QueryType::NominationDetails)
                }
                _ => false,
            },
        )
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    bot.save_or_restore_chat(chat_id).await.unwrap();
    for _ in 0..validator_count {
        let account_id = get_random_account_id();
        bot.network_postgres
            .add_validator_to_chat(chat_id, &account_id, &account_id.to_ss58_check(), &None)
            .await
            .unwrap();
    }
    bot.process_command(chat_id, "/nominationdetails", &[])
        .await
        .unwrap();
}
