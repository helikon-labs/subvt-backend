use crate::messenger::message::MessageType;
use crate::messenger::MockMessenger;
use crate::test::util::data::{get_telegram_bool_response, get_telegram_message_response};
use crate::test::util::{get_random_account_id, get_random_chat_id, new_test_bot};
use subvt_types::crypto::AccountId;

#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_nfts_no_validator() {
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
    assert!(bot.process_command(chat_id, "/nfts", &[]).await.is_ok());
}

#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_nfts_single_validator_no_nfts() {
    let chat_id = get_random_chat_id();
    let account_id = get_random_account_id();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::Loading)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    messenger
        .expect_delete_message()
        .returning(|_, _| Ok(get_telegram_bool_response(true)));
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::NoNFTsForValidator)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    bot.network_postgres
        .add_validator_to_chat(chat_id, &account_id, &account_id.to_ss58_check(), &None)
        .await
        .unwrap();
    assert!(bot.process_command(chat_id, "/nfts", &[]).await.is_ok());
}

#[tokio::test]
#[allow(clippy::borrowed_box)]
async fn test_nfts_single_validator_with_nfts() {
    let chat_id = get_random_chat_id();
    let account_id =
        AccountId::from_ss58_check("GC8fuEZG4E5epGf5KGXtcDfvrc6HXE7GJ5YnbiqSpqdQYLg").unwrap();
    let mut messenger = MockMessenger::new();
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::Loading)
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    messenger
        .expect_delete_message()
        .returning(|_, _| Ok(get_telegram_bool_response(true)));
    messenger
        .expect_send_message()
        .withf(|_, _, _, message_type: &Box<MessageType>| {
            matches!(**message_type, MessageType::NFTs { .. })
        })
        .returning(|_, _, _, _| Ok(get_telegram_message_response()));
    let bot = new_test_bot(messenger).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    bot.network_postgres
        .add_validator_to_chat(chat_id, &account_id, &account_id.to_ss58_check(), &None)
        .await
        .unwrap();
    assert!(bot.process_command(chat_id, "/nfts", &[]).await.is_ok());
}
