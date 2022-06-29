use crate::messenger::message::MessageType;
use crate::messenger::MockMessenger;
use crate::tests::util::{data::get_telegram_response_message, get_random_chat_id, new_test_bot};
use frankenstein::MethodResponse;

#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_about() {
    let chat_id = get_random_chat_id();
    let response = MethodResponse {
        ok: true,
        result: get_telegram_response_message(),
        description: None,
    };
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
