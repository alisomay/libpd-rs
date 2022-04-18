use libpd_rs::convenience::PdGlobal;

#[test]
fn open_close_patch() {
    let mut pd = PdGlobal::init_and_configure(0, 2, 44100).unwrap();

    assert!(pd.open_patch("tests/patches/sine.pd").is_ok());

    // Re-opens.
    assert!(pd.open_patch("tests/patches/sine.pd").is_ok());
    assert!(pd.close_patch().is_ok());

    assert!(pd.open_patch("non existent").is_err());
}
