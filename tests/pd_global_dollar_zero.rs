use libpd_rs::Pd;

#[test]
fn dollar_zero() {
    let mut pd = Pd::init_and_configure(0, 2, 44100).unwrap();
    pd.open_patch("tests/patches/sine.pd").unwrap();
    assert!(pd.dollar_zero().is_ok());
    assert_ne!(pd.dollar_zero().unwrap(), 0);
    pd.close_patch().unwrap();
    assert!(pd.dollar_zero().is_err());
}
