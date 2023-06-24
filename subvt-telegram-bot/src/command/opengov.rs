use crate::messenger::message::MessageType;
use crate::messenger::Messenger;
use crate::TelegramBot;
use enum_iterator::all;
use subvt_types::governance::track::Track;

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    //! Sends the user the payouts report for a selected validator. The report is a chart that
    //! displays the total paid out amount by the validator per month in the native token.
    pub(crate) async fn process_opengov_command(&self, chat_id: i64) -> anyhow::Result<()> {
        let tracks = all::<Track>().collect::<Vec<_>>();
        let mut data = vec![];
        for track in &tracks {
            let referenda = self
                .network_postgres
                .get_open_referenda(Some(track.id()))
                .await?;
            if !referenda.is_empty() {
                data.push((*track, referenda.len()));
            }
        }
        self.messenger
            .send_message(
                &self.app_postgres,
                &self.network_postgres,
                chat_id,
                Box::new(MessageType::ReferendumTracks(data)),
            )
            .await?;
        Ok(())
    }
}
