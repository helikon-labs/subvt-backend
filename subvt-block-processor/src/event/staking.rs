use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::event::staking::StakingEvent;

pub(crate) async fn process_staking_event(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    event_index: usize,
    event: &StakingEvent,
) -> anyhow::Result<()> {
    match event {
        StakingEvent::Chilled {
            extrinsic_index,
            stash_account_id,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_chilled_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    stash_account_id,
                )
                .await?;
        }
        StakingEvent::EraPaid {
            extrinsic_index,
            era_index,
            validator_payout,
            remainder,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_era_paid_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *era_index,
                    *validator_payout,
                    *remainder,
                )
                .await?;
        }
        StakingEvent::NominatorKicked {
            extrinsic_index,
            nominator_account_id,
            validator_account_id,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_nominator_kicked_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    validator_account_id,
                    nominator_account_id,
                )
                .await?;
        }
        StakingEvent::PayoutStarted {
            extrinsic_index,
            era_index,
            validator_account_id,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_payout_started_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *era_index,
                    validator_account_id,
                )
                .await?;
        }
        StakingEvent::Rewarded {
            extrinsic_index,
            rewardee_account_id,
            reward_destination: _reward_destination,
            amount,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_rewarded_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    rewardee_account_id,
                    *amount,
                )
                .await?;
        }
        StakingEvent::Slashed {
            extrinsic_index,
            validator_account_id,
            amount,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_slashed_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    validator_account_id,
                    *amount,
                )
                .await?;
        }
        _ => (),
    }
    Ok(())
}

pub(crate) async fn update_staking_event_nesting_index(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    maybe_nesting_index: &Option<String>,
    event_index: i32,
    event: &StakingEvent,
) -> anyhow::Result<()> {
    match event {
        StakingEvent::Chilled {
            stash_account_id, ..
        } => {
            postgres
                .update_chilled_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                    stash_account_id,
                )
                .await?;
        }
        StakingEvent::EraPaid { era_index, .. } => {
            postgres
                .update_era_paid_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                    *era_index,
                )
                .await?;
        }
        StakingEvent::NominatorKicked { .. } => {
            postgres
                .update_nominator_kicked_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
        StakingEvent::PayoutStarted { .. } => {
            postgres
                .update_payout_started_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
        StakingEvent::Rewarded { .. } => {
            postgres
                .update_rewarded_event_nesting_index(block_hash, maybe_nesting_index, event_index)
                .await?;
        }
        StakingEvent::Slashed { .. } => {
            postgres
                .update_slashed_event_nesting_index(block_hash, maybe_nesting_index, event_index)
                .await?;
        }
        _ => (),
    }
    Ok(())
}
