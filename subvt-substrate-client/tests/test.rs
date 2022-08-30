//! Substrate client tests. WiP.
use std::str::FromStr;
use subvt_config::Config;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::DemocracyVoting;

#[tokio::test]
async fn test_get_block_hash() {
    let config = Config::test().expect("Cannot get test config.");
    let substrate_client = SubstrateClient::new(&config)
        .await
        .expect("Cannot initialize client.");
    let block_number = 8_500_000;
    let hash = substrate_client
        .get_block_hash(block_number)
        .await
        .unwrap_or_else(|_| panic!("Cannot get block hash for block #{}.", block_number));
    let hash = hex::decode(hash.trim_start_matches("0x"))
        .unwrap_or_else(|_| panic!("Cannot decode hash from client."));
    let expected_hash =
        hex::decode("9d95763d4119488779991da8d1b16874687a3308ffcf9f89284d0382e8ccd161").unwrap();
    assert_eq!(hash, expected_hash);
}

#[tokio::test]
async fn test_get_democracy_voting_direct() {
    let config = Config::test().expect("Cannot get test config.");
    let substrate_client = SubstrateClient::new(&config)
        .await
        .expect("Cannot initialize client.");
    let account_id = AccountId::from_str("GC8fuEZG4E5epGf5KGXtcDfvrc6HXE7GJ5YnbiqSpqdQYLg")
        .expect("Cannot create account id.");
    let block_number = 14_229_000;
    let block_hash = substrate_client
        .get_block_hash(block_number)
        .await
        .unwrap_or_else(|_| panic!("Cannot get block hash for block #{}.", block_number));
    let voting = substrate_client
        .get_democracy_voting_of(&account_id, Some(&block_hash))
        .await
        .unwrap_or_else(|_| panic!("Error while trying to get democracy voting."))
        .unwrap();
    match voting {
        DemocracyVoting::Direct {
            votes,
            delegations: _delegations,
            prior: _prior,
        } => {
            println!("Direct voting. Vote count {}.", votes.len());
        }
        DemocracyVoting::Delegating {
            balance,
            target,
            conviction: _conviction,
            delegations: _delegations,
            prior: _prior,
        } => {
            panic!(
                "Unexpected delegated voting. Target {}. Balance {}.",
                target.to_ss58_check(),
                balance
            );
        }
    }
}

#[tokio::test]
async fn test_get_democracy_voting_delegated() {
    let config = Config::test().expect("Cannot get test config.");
    let substrate_client = SubstrateClient::new(&config)
        .await
        .expect("Cannot initialize client.");
    let account_id = AccountId::from_str("GpyTMuLmG3ADWRxhZpHQh5rqMgNpFoNUyxA1DJAXfvsQ2Ly")
        .expect("Cannot create account id.");
    let block_number = 14_229_000;
    let block_hash = substrate_client
        .get_block_hash(block_number)
        .await
        .unwrap_or_else(|_| panic!("Cannot get block hash for block #{}.", block_number));
    let voting = substrate_client
        .get_democracy_voting_of(&account_id, Some(&block_hash))
        .await
        .unwrap_or_else(|_| panic!("Error while trying to get democracy voting."))
        .unwrap();
    match voting {
        DemocracyVoting::Direct {
            votes,
            delegations: _delegations,
            prior: _prior,
        } => {
            log::debug!("Unexpected direct voting. Vote count {}.", votes.len());
            assert_eq!(10, votes.len());
        }
        DemocracyVoting::Delegating {
            balance,
            target,
            conviction: _conviction,
            delegations: _delegations,
            prior: _prior,
        } => {
            log::debug!(
                "Delegated voting. Target {}. Balance {}.",
                target.to_ss58_check(),
                balance
            );
            assert_eq!(
                "5CMiFyio1HrefWXAKB8kBmPn6dYa1SjJUwYYtyVXSPeys6nH",
                target.to_ss58_check()
            );
        }
    }
}
