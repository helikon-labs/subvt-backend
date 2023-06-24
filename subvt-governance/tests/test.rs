use subvt_governance::polkassembly::fetch_track_referenda;
use subvt_types::governance::track::Track;

#[tokio::test]
async fn test_fetch_small_spender_referenda() {
    let list = fetch_track_referenda(Track::SmallSpender.id(), 1, 100)
        .await
        .unwrap();
    for referendum in &list {
        assert_eq!(referendum.ty, "ReferendumV2");
    }
}
