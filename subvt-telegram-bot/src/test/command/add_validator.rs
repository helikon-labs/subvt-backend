use crate::messenger::MockMessenger;
use crate::test::util::data::{add_validator_to_redis, get_telegram_message_response};
use crate::test::util::{get_random_account_id, get_random_chat_id, new_test_bot};
use crate::MessageType;

/// Tests the case when the user enters the /add command without following it
/// by the stash address.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_add_validator_no_address() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::AddValidator)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot.process_command(chat_id, "/add", &[]).await.is_ok());
}

/// Tests the case when the user enters an invalid SS58 address following the /add command.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_add_validator_invalid_address() {
    let chat_id = get_random_chat_id();
    let invalid_address = "ABC";
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(
            move |_, _, _, message_type: &Box<MessageType>| match &**message_type {
                MessageType::InvalidAddress(address) => address == invalid_address,
                _ => false,
            },
        )
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot
        .process_command(chat_id, "/add", &[invalid_address.to_string()])
        .await
        .is_ok());
}

/// Tests when the user tries to add a validator that doesn't exist in the Redis database,
/// which is updated by the SubVT validator list updater component.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_add_non_existent_validator() {
    let chat_id = get_random_chat_id();
    let non_existent_address = get_random_account_id().to_ss58_check();
    let command_args = [non_existent_address.clone()];
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(
            move |_, _, _, message_type: &Box<MessageType>| match &**message_type {
                MessageType::AddValidatorNotFound(address) => address == &non_existent_address,
                _ => false,
            },
        )
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot
        .process_command(chat_id, "/add", &command_args)
        .await
        .is_ok());
}

/// Tests the case of trying to add a validator that already exists in the chat.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_add_validator_duplicate() {
    let chat_id = get_random_chat_id();
    let account_id = get_random_account_id();
    let address = account_id.to_ss58_check();
    let command_args = [address.clone()];
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(move |_, _, _, message_type: &Box<MessageType>| {
            matches!(
                &**message_type,
                MessageType::ValidatorExistsOnChat(_duplicate_address)
            )
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    add_validator_to_redis(&bot.redis, &account_id)
        .await
        .unwrap();
    bot.network_postgres
        .add_validator_to_chat(chat_id, &account_id, &account_id.to_ss58_check(), &None)
        .await
        .unwrap();
    assert!(bot
        .process_command(chat_id, "/add", &command_args)
        .await
        .is_ok());
}

/// Test the successful addition of a validator to a chat.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_add_validator_successful() {
    let chat_id = get_random_chat_id();
    let account_id = get_random_account_id();
    let address = account_id.to_ss58_check();
    let command_args = [address.clone()];
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(
            move |_, _, _, message_type: &Box<MessageType>| match &**message_type {
                MessageType::ValidatorInfo {
                    address: added_validator_address,
                    ..
                } => added_validator_address == address.as_str(),
                _ => false,
            },
        )
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::ValidatorAdded)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    add_validator_to_redis(&bot.redis, &account_id)
        .await
        .unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot
        .process_command(chat_id, "/add", &command_args)
        .await
        .is_ok());
}
