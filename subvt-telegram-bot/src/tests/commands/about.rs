use super::super::util::data::get_telegram_response_message;
use crate::messenger::message::MessageType;
use crate::tests::new_test_bot;
use crate::tests::MockMessenger;
use frankenstein::MethodResponse;

#[allow(clippy::borrowed_box)]
#[tokio::test]
async fn test_about() {
    let response = MethodResponse {
        ok: true,
        result: get_telegram_response_message(),
        description: None,
    };
    let chat_id = 1;
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::About)
        })
        .return_once(move |_, _, _, _| Ok(response));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot.process_command(chat_id, "/about", &[]).await.is_ok());
}
