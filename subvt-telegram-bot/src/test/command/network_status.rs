use crate::messenger::MockMessenger;
use crate::test::util::data::get_telegram_message_response;
use crate::test::util::{get_random_chat_id, new_test_bot};
use crate::MessageType;
use subvt_types::subvt::NetworkStatus;

/// Tests the successsful /network command (alias /networkstatus), which displays
/// network status data such as staking values, block heights, etc.
#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_get_network_status_success() {
    let chat_id = get_random_chat_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::NetworkStatus(_))
        })
        .times(2)
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    bot.redis
        .set_network_status(&NetworkStatus::default())
        .await
        .unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot.process_command(chat_id, "/network", &[]).await.is_ok());
    assert!(bot
        .process_command(chat_id, "/networkstatus", &[])
        .await
        .is_ok());
}
