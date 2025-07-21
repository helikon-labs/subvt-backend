use crate::messenger::MockMessenger;
use crate::query::QueryType;
use crate::test::util::data::get_telegram_message_response;
use crate::test::util::{get_random_account_id, get_random_chat_id, new_test_bot};
use crate::MessageType;
use rand::Rng;

/// Tests /remove command with no validator added to the chat.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_remove_no_validator() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::NoValidatorsOnChat)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    bot.save_or_restore_chat(chat_id).await.unwrap();
    bot.process_command(chat_id, "/remove", &[]).await.unwrap();
}

/// Tests /remove command with a single validator added to the chat.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_remove_single_validator() {
    let chat_id = get_random_chat_id();
    let account_id = get_random_account_id();
    let address = account_id.to_ss58_check();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(
            move |_, _, _, message_type: &Box<MessageType>| match &**message_type {
                MessageType::ValidatorRemoved(validator) => validator.address == address,
                _ => false,
            },
        )
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    bot.save_or_restore_chat(chat_id).await.unwrap();
    bot.network_postgres
        .add_validator_to_chat(chat_id, &account_id, &account_id.to_ss58_check(), &None)
        .await
        .unwrap();
    bot.process_command(chat_id, "/remove", &[]).await.unwrap();
}

/// Tests /remove command with multiple validators on the chat - user should receive
/// a list of validators to pick the one to remove.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_remove_multiple_validators() {
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
                        && matches!(query_type, QueryType::RemoveValidator)
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
    bot.process_command(chat_id, "/remove", &[]).await.unwrap();
}
