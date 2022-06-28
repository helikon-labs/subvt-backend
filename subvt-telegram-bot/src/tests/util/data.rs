use frankenstein::{Chat, ChatType, Message};

pub fn get_telegram_response_message() -> Message {
    let chat = Chat::builder().id(0).type_field(ChatType::Private).build();
    Message::builder()
        .message_id(0)
        .date(0)
        .chat(chat)
        //.text(Option::<String>::None) // Option<String>
        //.from(None)
        //.sender_chat(None)
        //.group_chat_created(None) // Option(Bool)
        //.supergroup_chat_created(None) // Option(Bool)
        .build()
}
