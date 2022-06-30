use crate::messenger::MockMessenger;
use crate::query::QueryType;
use crate::test::util::data::{add_validator_to_redis, get_telegram_message_response};
use crate::test::util::{get_random_account_id, get_random_chat_id, new_test_bot};
use crate::MessageType;
use rand::Rng;

/// Tests the case when the user calls the /payouts command before adding
/// any validators to the chat.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_payouts_no_validator() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::NoValidatorsOnChat)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot.process_command(chat_id, "/payouts", &[]).await.is_ok());
}

/// Tests calling the /payouts command with a single validator on the chat that has no
/// payouts yet.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_payouts_single_validator_no_payouts() {
    let chat_id = get_random_chat_id();
    let account_id = get_random_account_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(&**message_type, MessageType::NoPayoutsFound)
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
    assert!(bot.process_command(chat_id, "/payouts", &[]).await.is_ok());
}

/// Tests calling the /payouts command with multiple single validator on the chat - the
/// user should receive the list of validators to pick one from.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_payouts_multiple_validators() {
    let mut rng = rand::thread_rng();
    let validator_count = rng.gen_range(3..15);
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
                    validators.len() == validator_count && matches!(query_type, QueryType::Payouts)
                }
                _ => false,
            },
        )
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    for _ in 0..validator_count {
        let account_id = get_random_account_id();
        bot.network_postgres
            .add_validator_to_chat(chat_id, &account_id, &account_id.to_ss58_check(), &None)
            .await
            .unwrap();
    }
    assert!(bot.process_command(chat_id, "/payouts", &[]).await.is_ok());
}
