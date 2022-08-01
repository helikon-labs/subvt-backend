use crate::messenger::MockMessenger;
use crate::test::util::data::get_telegram_message_response;
use crate::test::util::{get_random_chat_id, new_test_bot};
use crate::MessageType;

/// Tests the /democracy command, which gives a list of all open referenda to the user.
/// The response could be that there are no open referenda.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_democracy() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::NoOpenReferendaFound)
                || matches!(**message_type, MessageType::RefererendumList(_))
        })
        .times(2)
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    bot.save_or_restore_chat(chat_id).await.unwrap();
    bot.process_command(chat_id, "/democracy", &[])
        .await
        .unwrap();
    bot.process_command(chat_id, "/referenda", &[])
        .await
        .unwrap();
}
