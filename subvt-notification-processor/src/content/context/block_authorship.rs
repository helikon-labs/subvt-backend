use subvt_types::app::{Block, Notification};
use tera::Context;

pub(crate) fn set_block_authorship_context(
    notification: &Notification,
    context: &mut Context,
) -> anyhow::Result<()> {
    let block: Block = serde_json::from_str(notification.data_json.as_ref().unwrap())?;
    context.insert("block_number", &block.number);
    Ok(())
}
