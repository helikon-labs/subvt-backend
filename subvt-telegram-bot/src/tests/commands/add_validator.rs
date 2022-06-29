use frankenstein::MethodResponse;
use crate::MessageType;
use crate::messenger::MockMessenger;
use crate::tests::util::{get_random_chat_id, new_test_bot};
use crate::tests::util::data::get_telegram_response_message;

#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_add_non_existent_validator() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    let response = MethodResponse {
        ok: true,
        result: get_telegram_response_message(),
        description: None,
    };
    messenger
        .expect_send_message()
        .withf(move |_, _, _, message_type: &Box<MessageType>| {
            match &**message_type {
                MessageType::InvalidAddress(account_id) => account_id == "0xABC",
                _ => false
            }
        })
        .return_once(move |_, _, _, _| Ok(response));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(
        bot.process_command(
            chat_id,
            "/add",
            &["0xABC".to_string()]
        ).await.is_ok()
    );
}
