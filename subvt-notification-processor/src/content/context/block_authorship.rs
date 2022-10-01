use subvt_types::app::{notification::Notification, Block};
use tera::Context;

pub(crate) fn set_block_authorship_grouped_context(
    notifications: &[Notification],
    context: &mut Context,
) -> anyhow::Result<()> {
    let mut block_numbers = vec![];
    for notification in notifications {
        let block: Block = serde_json::from_str(notification.data_json.as_ref().unwrap())?;
        block_numbers.push(block.number);
    }
    block_numbers.sort_unstable();
    context.insert("block_numbers", &block_numbers);
    Ok(())
}

pub(crate) fn set_block_authorship_context(
    notification: &Notification,
    context: &mut Context,
) -> anyhow::Result<()> {
    let block: Block = serde_json::from_str(notification.data_json.as_ref().unwrap())?;
    context.insert("block_number", &block.number);
    Ok(())
}
