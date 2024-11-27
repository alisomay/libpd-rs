use libpd_rs::Pd;

#[test]
fn subscribe_unsubscribe() {
    let mut pd = Pd::init_and_configure(0, 2, 44100).unwrap();
    assert!(pd.open_patch("tests/patches/sine.pd").is_ok());
    assert!(pd.subscribe_to("a_source").is_ok());
    assert!(pd.subscriptions.contains_key("a_source"));
    assert!(pd.subscribe_to_many(&["other", "another"]).is_ok());
    assert!(pd.subscriptions.contains_key("a_source"));
    assert!(pd.subscriptions.contains_key("other"));
    assert!(pd.subscriptions.contains_key("another"));
    pd.unsubscribe_from("a_source");
    pd.unsubscribe_from("a_source");
    assert!(!pd.subscriptions.contains_key("a_source"));
    pd.unsubscribe_from_all();
    assert!(!pd.subscriptions.contains_key("a_source"));
    assert!(!pd.subscriptions.contains_key("other"));
    assert!(!pd.subscriptions.contains_key("another"));
}
