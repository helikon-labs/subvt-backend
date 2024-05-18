//! Substrate client tests. WiP.
use std::str::FromStr;
use subvt_config::Config;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::governance::track::Track;
use subvt_types::substrate::{Chain, ConvictionVoting};

#[tokio::test]
async fn test_get_block_hash() {
    let config = Config::test().expect("Cannot get test config.");
    let substrate_client = SubstrateClient::new(
        config.substrate.rpc_url.as_str(),
        config.substrate.network_id,
        config.substrate.connection_timeout_seconds,
        config.substrate.request_timeout_seconds,
    )
    .await
    .expect("Cannot initialize client.");
    let block_number = 8_500_000;
    let hash = substrate_client
        .get_block_hash(block_number)
        .await
        .unwrap_or_else(|_| panic!("Cannot get block hash for block #{block_number}."));
    let hash = hex::decode(hash.trim_start_matches("0x"))
        .unwrap_or_else(|_| panic!("Cannot decode hash from client."));
    let expected_hash =
        hex::decode("9d95763d4119488779991da8d1b16874687a3308ffcf9f89284d0382e8ccd161").unwrap();
    assert_eq!(hash, expected_hash);
}

#[tokio::test]
async fn test_get_conviction_voting_direct() {
    let config = Config::test().expect("Cannot get test config.");
    let substrate_client = SubstrateClient::new(
        config.substrate.rpc_url.as_str(),
        config.substrate.network_id,
        config.substrate.connection_timeout_seconds,
        config.substrate.request_timeout_seconds,
    )
    .await
    .expect("Cannot initialize client.");
    let account_id = AccountId::from_str("GC8fuEZG4E5epGf5KGXtcDfvrc6HXE7GJ5YnbiqSpqdQYLg")
        .expect("Cannot create account id.");
    let block_number = 18_497_450;
    let block_hash = substrate_client
        .get_block_hash(block_number)
        .await
        .unwrap_or_else(|_| panic!("Cannot get block hash for block #{block_number}."));
    let voting = substrate_client
        .get_conviction_voting_for(&account_id, Track::MediumSpender.id(), Some(&block_hash))
        .await
        .unwrap_or_else(|_| panic!("Error while trying to get democracy voting."))
        .unwrap();
    match voting {
        ConvictionVoting::Casting(casting) => {
            println!("Direct voting. Vote count {}.", casting.votes.len());
        }
        ConvictionVoting::Delegating(delegating) => {
            panic!(
                "Unexpected delegated voting. Target {}. Balance {}.",
                delegating.target.to_ss58_check(),
                delegating.balance
            );
        }
    }
}

#[tokio::test]
async fn test_get_conviction_voting_delegated() {
    Chain::Kusama.sp_core_set_default_ss58_version();
    let config = Config::test().expect("Cannot get test config.");
    let substrate_client = SubstrateClient::new(
        config.substrate.rpc_url.as_str(),
        config.substrate.network_id,
        config.substrate.connection_timeout_seconds,
        config.substrate.request_timeout_seconds,
    )
    .await
    .expect("Cannot initialize client.");
    let account_id = AccountId::from_str("GkE7EHZNtcPoqYSBqB8r9n5GYHPwTFcsSpKzzsFo3ZmHCBk")
        .expect("Cannot create account id.");
    let block_number = 18_497_450;
    let block_hash = substrate_client
        .get_block_hash(block_number)
        .await
        .unwrap_or_else(|_| panic!("Cannot get block hash for block #{block_number}."));
    let voting = substrate_client
        .get_conviction_voting_for(&account_id, Track::MediumSpender.id(), Some(&block_hash))
        .await
        .unwrap_or_else(|_| panic!("Error while trying to get democracy voting."))
        .unwrap();
    match voting {
        ConvictionVoting::Casting(casting) => {
            panic!(
                "Unexpected direct voting. Vote count {}.",
                casting.votes.len()
            );
        }
        ConvictionVoting::Delegating(delegating) => {
            println!(
                "Delegated voting. Target {}. Balance {}. Conviction {:?}.",
                delegating.target.to_ss58_check(),
                delegating.balance,
                delegating.conviction,
            );
            assert_eq!(
                "HvshspvW9yk29mrCSBLcpv8MRbna4incDkt5C86ezZ6XKPH",
                delegating.target.to_ss58_check(),
            );
        }
    }
}

#[tokio::test]
async fn test_get_conviction_referendum_voting_direct() {
    let config = Config::test().expect("Cannot get test config.");
    let substrate_client = SubstrateClient::new(
        config.substrate.rpc_url.as_str(),
        config.substrate.network_id,
        config.substrate.connection_timeout_seconds,
        config.substrate.request_timeout_seconds,
    )
    .await
    .expect("Cannot initialize client.");
    let account_id = AccountId::from_str("GC8fuEZG4E5epGf5KGXtcDfvrc6HXE7GJ5YnbiqSpqdQYLg")
        .expect("Cannot create account id.");
    let block_number = 18_498_121;
    let block_hash = substrate_client
        .get_block_hash(block_number)
        .await
        .unwrap_or_else(|_| panic!("Cannot get block hash for block #{block_number}."));
    let vote = substrate_client
        .get_account_referendum_conviction_vote(
            &account_id,
            Track::MediumSpender.id(),
            220,
            Some(&block_hash),
        )
        .await
        .unwrap()
        .unwrap();
    assert!(vote.direct_vote.is_some());
    assert!(vote.direct_vote.unwrap().conviction.is_some());
    assert!(vote.direct_vote.unwrap().nay.is_some());
}
