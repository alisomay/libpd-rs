use libpd_rs::convenience::PdGlobal;

#[test]
fn subscribe_unsubscribe() {
    let mut pd = PdGlobal::init_and_configure(0, 2, 44100).unwrap();
    assert!(pd.open_patch("tests/patches/sine.pd").is_ok());
    assert!(pd.subscribe_to("a_source").is_ok());
    assert!(pd.subscriptions.get("a_source").is_some());
    assert!(pd.subscribe_to_many(&["other", "another"]).is_ok());
    assert!(pd.subscriptions.get("a_source").is_some());
    assert!(pd.subscriptions.get("other").is_some());
    assert!(pd.subscriptions.get("another").is_some());
    pd.unsubscribe_from("a_source");
    pd.unsubscribe_from("a_source");
    assert!(pd.subscriptions.get("a_source").is_none());
    pd.unsubscribe_from_all();
    assert!(pd.subscriptions.get("a_source").is_none());
    assert!(pd.subscriptions.get("other").is_none());
    assert!(pd.subscriptions.get("another").is_none());
}
