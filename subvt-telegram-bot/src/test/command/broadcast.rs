use crate::messenger::message::MessageType;
use crate::messenger::MockMessenger;
use crate::test::util::data::get_telegram_message_response;
use crate::test::util::{get_random_chat_id, new_test_bot};

static ADMIN_CHAT_ID: i64 = 1234567890;

/// /broadcast command can only be called by the chat admin. This function tests the successful
/// result of the call.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_broadcasttest() {
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::Broadcast)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(ADMIN_CHAT_ID).await.is_ok());
    assert!(bot
        .process_command(ADMIN_CHAT_ID, "/broadcasttest", &[])
        .await
        .is_ok());
}

/// Tests the case of calling the /broadcast and /broadcasttest commands from an
/// unauthorized chat.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_broadcast_and_broadcasttest_non_admin() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::UnknownCommand(_))
        })
        .times(2)
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot
        .process_command(chat_id, "/broadcast", &[])
        .await
        .is_ok());
    assert!(bot
        .process_command(chat_id, "/broadcasttest", &[])
        .await
        .is_ok());
}

/// Test the calling of the /broadcast command by an authorized chat. This command is replied
/// with a confirmation message that asks whether the admin is sure about broadcasting the
/// message to all chats.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_broadcast() {
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::BroadcastConfirm)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(ADMIN_CHAT_ID).await.is_ok());
    assert!(bot
        .process_command(ADMIN_CHAT_ID, "/broadcast", &[])
        .await
        .is_ok());
}
