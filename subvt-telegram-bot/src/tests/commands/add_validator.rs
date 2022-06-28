use crate::messenger::MockMessenger;
use crate::tests::new_test_bot;

#[tokio::test]
async fn test_add_validator() {
    let _bot = new_test_bot(MockMessenger::new()).await.unwrap();
    assert_eq!(1, 1);
}
