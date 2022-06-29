use crate::messenger::MockMessenger;
use crate::test::util::{get_random_chat_id, new_test_bot};
use crate::{MessengerImpl, TelegramBot, DEFAULT_RULES};

#[tokio::test]
async fn test_save_new_chat() {
    let chat_id = get_random_chat_id();
    let bot = new_test_bot(MockMessenger::new()).await.unwrap();
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot
        .network_postgres
        .chat_exists_by_id(chat_id)
        .await
        .unwrap());
    let user_id = bot
        .network_postgres
        .get_chat_app_user_id(chat_id)
        .await
        .unwrap();
    let channels = bot
        .app_postgres
        .get_user_notification_channels(user_id)
        .await
        .unwrap();
    assert_eq!(1, channels.len());
    assert_eq!(chat_id.to_string(), channels[0].target);
    assert_eq!(
        DEFAULT_RULES.len(),
        bot.app_postgres
            .get_user_notification_rules(user_id)
            .await
            .unwrap()
            .len()
    );
}

#[tokio::test]
async fn test_restore_chat_and_user() {
    let bot: TelegramBot<MessengerImpl> = TelegramBot::<MessengerImpl>::new().await.unwrap();
    let chat_id = 2;
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    let user_id = bot
        .network_postgres
        .get_chat_app_user_id(chat_id)
        .await
        .unwrap();
    assert!(!bot.network_postgres.chat_is_deleted(chat_id).await.unwrap());
    assert!(bot.network_postgres.delete_chat(chat_id).await.is_ok());
    assert!(bot.app_postgres.delete_user(user_id).await.is_ok());
    assert!(bot.network_postgres.chat_is_deleted(chat_id).await.unwrap());
    assert!(!bot.app_postgres.user_exists_by_id(user_id).await.unwrap());
    assert!(bot.save_or_restore_chat(chat_id).await.is_ok());
    assert!(bot
        .network_postgres
        .chat_exists_by_id(chat_id)
        .await
        .unwrap());
    assert!(bot.app_postgres.user_exists_by_id(user_id).await.unwrap());
    assert_eq!(
        DEFAULT_RULES.len(),
        bot.app_postgres
            .get_user_notification_rules(user_id)
            .await
            .unwrap()
            .len()
    );
}
