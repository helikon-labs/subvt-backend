use crate::messenger::message::MessageType;
use crate::messenger::Messenger;
use crate::query::Query;
use crate::TelegramBot;
use subvt_types::governance::track::Track;

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    pub(crate) async fn process_referendum_track_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
    ) -> anyhow::Result<()> {
        if let Some(message_id) = original_message_id {
            self.messenger.delete_message(chat_id, message_id).await?;
        }
        if let Some(track_id_str) = &query.parameter {
            if let Ok(track_id) = track_id_str.parse::<u16>() {
                let posts = self
                    .network_postgres
                    .get_open_referenda(Some(track_id))
                    .await?;
                if posts.is_empty() {
                    if let Some(track) = Track::from_id(track_id) {
                        self.messenger
                            .send_message(
                                &self.app_postgres,
                                &self.network_postgres,
                                chat_id,
                                Box::new(MessageType::NoOpenReferendaFound(track)),
                            )
                            .await?;
                    }
                } else {
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::ReferendumList(track_id, posts)),
                        )
                        .await?;
                }
            }
        }
        Ok(())
    }
}
