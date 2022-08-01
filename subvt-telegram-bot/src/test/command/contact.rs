use crate::messenger::MockMessenger;
use crate::test::util::data::get_telegram_message_response;
use crate::test::util::{get_random_chat_id, new_test_bot};
use crate::MessageType;

/// Tests the initial reply to the /contact command, which helps the user to submit
/// bug reports and feature requests.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_contact() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::SelectContactType)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    bot.save_or_restore_chat(chat_id).await.unwrap();
    bot.process_command(chat_id, "/contact", &[]).await.unwrap();
}
