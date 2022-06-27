use crate::TelegramBot;

#[tokio::test]
async fn test_init() {
    let bot_result = TelegramBot::new().await;
    assert!(bot_result.is_ok());
    let _bot = bot_result.unwrap();
}
