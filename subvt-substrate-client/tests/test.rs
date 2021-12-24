use subvt_config::Config;
use subvt_substrate_client::SubstrateClient;

#[tokio::test]
async fn test_get_block_hash() {
    let config = Config::test().expect("Cannot get test config.");
    let substrate_client = SubstrateClient::new(&config)
        .await
        .expect("Cannot initialize client.");
    let block_number = 8500000;
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
