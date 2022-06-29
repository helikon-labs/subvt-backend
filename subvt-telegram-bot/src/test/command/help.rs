use crate::messenger::message::MessageType;
use crate::messenger::MockMessenger;
use crate::test::util::data::get_telegram_response;
use crate::test::util::{get_random_chat_id, new_test_bot};

#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_help() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::Help)
        })
        .return_once(move |_, _, _, _| Ok(get_telegram_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot.process_command(chat_id, "/help", &[]).await.is_ok());
}
